use crate::error::AppError;
use crate::state::AppState;
use axum::{
    extract::{FromRequestParts, Request, State},
    http::{header, request::Parts, Extensions, HeaderMap, Method, Uri},
    middleware::Next,
    response::Response,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

const PBKDF2_ITERATIONS: u32 = 15_000;
const RECOVERY_ALPHABET: &[u8; 32] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub client_ip: String,
    pub user_agent: Option<String>,
    pub host: String,
    pub port: Option<u16>,
    pub is_https: bool,
    pub effective_rp_id: String,
    pub origin: Option<String>,
}

impl RequestContext {
    pub fn from_raw(
        headers: &HeaderMap,
        uri: &Uri,
        extensions: &Extensions,
        configured_rp_id: &str,
    ) -> Self {
        let client_ip = extract_sanitized_ip(headers, Some(extensions));
        let user_agent = extract_sanitized_user_agent(headers);
        let (host, port) = extract_sanitized_host_port(headers, Some(uri));
        let is_https = detect_is_https(headers, Some(uri));
        let origin = extract_sanitized_origin(headers);

        let mut effective_rp_id = compute_effective_rp_id(configured_rp_id, &host);
        if (effective_rp_id == "localhost" || effective_rp_id == "127.0.0.1") && origin.is_some() {
            if let Some(ref orig) = origin {
                if let Some((_, orig_host, _)) = parse_origin_scheme_host_port(orig) {
                    let from_orig = compute_effective_rp_id(configured_rp_id, orig_host);
                    if from_orig != "localhost" && from_orig != "127.0.0.1" {
                        effective_rp_id = from_orig;
                    }
                }
            }
        }

        Self {
            client_ip,
            user_agent,
            host,
            port,
            is_https,
            effective_rp_id,
            origin,
        }
    }

    pub fn from_request_parts(parts: &Parts, configured_rp_id: &str) -> Self {
        Self::from_raw(
            &parts.headers,
            &parts.uri,
            &parts.extensions,
            configured_rp_id,
        )
    }
}

impl<S> FromRequestParts<S> for RequestContext
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(ctx) = parts.extensions.get::<RequestContext>() {
            return Ok(ctx.clone());
        }

        let configured_rp_id = parts
            .extensions
            .get::<AppState>()
            .map(|st| st.config.rp_id.as_str())
            .unwrap_or("localhost");

        Ok(RequestContext::from_request_parts(parts, configured_rp_id))
    }
}

fn parse_ip_candidate(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(ip) = trimmed.parse::<IpAddr>() {
        return Some(ip.to_string());
    }
    if let Ok(socket) = trimmed.parse::<SocketAddr>() {
        return Some(socket.ip().to_string());
    }
    let unbracketed = trimmed.trim_matches(|c| c == '[' || c == ']');
    if let Ok(ip) = unbracketed.parse::<IpAddr>() {
        return Some(ip.to_string());
    }
    None
}

/// Extracts validated IPv4 or IPv6 client IP address:
/// checks first hop of X-Forwarded-For, then X-Real-IP, then ConnectInfo, fallback "127.0.0.1".
pub fn extract_sanitized_ip(headers: &HeaderMap, extensions: Option<&Extensions>) -> String {
    if let Some(val) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        if let Some(first) = val.split(',').next() {
            if let Some(ip) = parse_ip_candidate(first) {
                return ip;
            }
        }
    }

    if let Some(val) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
        if let Some(ip) = parse_ip_candidate(val) {
            return ip;
        }
    }

    if let Some(ext) = extensions {
        if let Some(connect_info) = ext.get::<axum::extract::ConnectInfo<SocketAddr>>() {
            return connect_info.0.ip().to_string();
        }
    }

    "127.0.0.1".to_string()
}

/// Extracts sanitized User-Agent: strips control characters, limits length to 512 chars.
pub fn extract_sanitized_user_agent(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(header::USER_AGENT)?.to_str().ok()?;
    let filtered: String = raw.chars().filter(|c| !c.is_control()).take(512).collect();
    let trimmed = filtered.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub fn parse_host_port(authority: &str) -> Option<(&str, Option<u16>)> {
    let trimmed = authority.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix('[') {
        let (ipv6, after) = rest.split_once(']')?;
        if let Some(port_str) = after.strip_prefix(':') {
            let port = port_str.parse::<u16>().ok()?;
            return Some((ipv6, Some(port)));
        } else if after.is_empty() {
            return Some((ipv6, None));
        } else {
            return None;
        }
    }
    if let Some((host, port_str)) = trimmed.rsplit_once(':') {
        let port = port_str.parse::<u16>().ok()?;
        if host.is_empty() {
            return None;
        }
        Some((host, Some(port)))
    } else {
        Some((trimmed, None))
    }
}

/// Extracts effective lowercase host and optional port from X-Forwarded-Host, Host header, or URI.
pub fn extract_sanitized_host_port(
    headers: &HeaderMap,
    uri: Option<&Uri>,
) -> (String, Option<u16>) {
    let raw_target = headers
        .get("x-forwarded-host")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or_else(|| {
            headers
                .get(header::HOST)
                .and_then(|v| v.to_str().ok())
                .map(str::trim)
                .filter(|s| !s.is_empty())
        })
        .or_else(|| uri.and_then(|u| u.authority()).map(|a| a.as_str()));

    if let Some(raw) = raw_target {
        let cleaned: String = raw.chars().filter(|c| !c.is_control()).collect();
        if let Some((h, p)) = parse_host_port(&cleaned) {
            let host_clean = h.trim().to_ascii_lowercase();
            if !host_clean.is_empty() {
                return (host_clean, p);
            }
        }
    }

    ("localhost".to_string(), None)
}

/// Detects if request is HTTPS:
/// checks X-Forwarded-Proto == "https", X-Forwarded-Ssl == "on", or URI scheme == "https".
pub fn detect_is_https(headers: &HeaderMap, uri: Option<&Uri>) -> bool {
    if let Some(proto) = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(str::trim)
    {
        if proto.eq_ignore_ascii_case("https") {
            return true;
        }
    }

    if let Some(ssl) = headers
        .get("x-forwarded-ssl")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(str::trim)
    {
        if ssl.eq_ignore_ascii_case("on") {
            return true;
        }
    }

    if let Some(uri) = uri {
        if let Some(scheme) = uri.scheme_str() {
            if scheme.eq_ignore_ascii_case("https") {
                return true;
            }
        }
    }

    false
}

/// Computes effective Relying Party ID (RP ID):
/// If the incoming request host is local (localhost, 127.0.0.1, ::1), always returns "localhost".
/// Otherwise respects non-local `configured_rp_id` or non-local request `host`.
pub fn compute_effective_rp_id(configured_rp_id: &str, host: &str) -> String {
    let host_trimmed = host.trim();
    if host_trimmed.eq_ignore_ascii_case("localhost")
        || host_trimmed == "127.0.0.1"
        || host_trimmed == "::1"
    {
        return "localhost".to_string();
    }

    let configured_trimmed = configured_rp_id.trim();
    if !configured_trimmed.is_empty()
        && !configured_trimmed.eq_ignore_ascii_case("localhost")
        && configured_trimmed != "127.0.0.1"
    {
        return configured_trimmed.to_string();
    }

    if !host_trimmed.is_empty() {
        return host_trimmed.to_string();
    }

    "localhost".to_string()
}

/// Extracts sanitized Origin header: strips control chars, trailing slashes, max 512 chars.
pub fn extract_sanitized_origin(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(header::ORIGIN)?.to_str().ok()?;
    let filtered: String = raw.chars().filter(|c| !c.is_control()).take(512).collect();
    let trimmed = filtered.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub fn parse_origin_scheme_host_port(origin: &str) -> Option<(&str, &str, Option<u16>)> {
    let (scheme, rest) = origin.trim().split_once("://")?;
    if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
        return None;
    }
    let authority = rest.split('/').next().unwrap_or(rest);
    let (host, port) = parse_host_port(authority)?;
    Some((scheme, host, port))
}

/// Checks whether an Origin matches the RequestContext (same scheme, host, and port).
pub fn is_same_origin(origin: &str, ctx: &RequestContext) -> bool {
    let Some((origin_scheme, origin_host, origin_port)) = parse_origin_scheme_host_port(origin)
    else {
        return false;
    };

    if !origin_host.eq_ignore_ascii_case(&ctx.host) {
        return false;
    }

    if ctx.is_https {
        if !origin_scheme.eq_ignore_ascii_case("https") {
            return false;
        }
    } else if !origin_scheme.eq_ignore_ascii_case("http") {
        return false;
    }

    let origin_port_canonical = match (origin_scheme.to_ascii_lowercase().as_str(), origin_port) {
        ("http", Some(80)) => None,
        ("https", Some(443)) => None,
        (_, p) => p,
    };

    let ctx_port_canonical = match (ctx.is_https, ctx.port) {
        (false, Some(80)) => None,
        (true, Some(443)) => None,
        (_, p) => p,
    };

    origin_port_canonical == ctx_port_canonical
}

/// Resolves the effective Relying Party ID (RP ID) for WebAuthn passkey operations.
/// Backwards-compatible helper that delegates to `compute_effective_rp_id`.
pub fn resolve_effective_rp_id(headers: &HeaderMap, configured_rp_id: &str) -> String {
    let (host, _) = extract_sanitized_host_port(headers, None);
    let mut rp_id = compute_effective_rp_id(configured_rp_id, &host);
    if rp_id == "localhost" || rp_id == "127.0.0.1" {
        if let Some(orig) = extract_sanitized_origin(headers) {
            if let Some((_, orig_host, _)) = parse_origin_scheme_host_port(&orig) {
                let from_orig = compute_effective_rp_id(configured_rp_id, orig_host);
                if from_orig != "localhost" && from_orig != "127.0.0.1" {
                    rp_id = from_orig;
                }
            }
        }
    }
    rp_id
}

/// Global infallible middleware that constructs `RequestContext` and inserts it into request extensions.
pub async fn request_context_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let ctx = RequestContext::from_raw(
        req.headers(),
        req.uri(),
        req.extensions(),
        &state.config.rp_id,
    );
    req.extensions_mut().insert(ctx);
    next.run(req).await
}

/// Enforces CSRF & Origin protection on all state-changing HTTP methods (`POST`, `PUT`, `PATCH`, `DELETE`)
/// as specified in `17-api-security-operations.md §3.2` (resolves **TD-002**):
/// 1. Requires `X-Requested-With: genealogy` header (case-insensitive value check).
/// 2. When `Origin` header is present, it must match one of the configured origins in `state.config.app_origin`,
///    or match the request's effective origin (`is_same_origin`), or `http://localhost:*` in dev/test.
pub async fn csrf_protection_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let method = req.method();
    let is_mutating = matches!(
        *method,
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    );

    if is_mutating {
        let xrw = req
            .headers()
            .get("x-requested-with")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !xrw.eq_ignore_ascii_case("genealogy") {
            return Err(AppError::CsrfValidationFailed);
        }

        if let Some(origin_val) = req.headers().get("origin") {
            let origin = origin_val
                .to_str()
                .unwrap_or("")
                .trim()
                .trim_end_matches('/');
            if !origin.is_empty() {
                let matches_configured = state
                    .config
                    .app_origin
                    .split([',', ';'])
                    .map(|s| s.trim().trim_end_matches('/'))
                    .any(|exp| !exp.is_empty() && exp.eq_ignore_ascii_case(origin));

                let is_localhost_dev = state.config.app_env != "production"
                    && (origin.starts_with("http://localhost:")
                        || origin.eq_ignore_ascii_case("http://localhost")
                        || origin.starts_with("http://127.0.0.1:")
                        || origin.eq_ignore_ascii_case("http://127.0.0.1"));

                let matches_request_host =
                    if let Some(ctx) = req.extensions().get::<RequestContext>() {
                        is_same_origin(origin, ctx)
                    } else {
                        let fallback_ctx = RequestContext::from_raw(
                            req.headers(),
                            req.uri(),
                            req.extensions(),
                            &state.config.rp_id,
                        );
                        is_same_origin(origin, &fallback_ctx)
                    };

                if !matches_configured && !is_localhost_dev && !matches_request_host {
                    return Err(AppError::CsrfValidationFailed);
                }
            }
        }
    }

    Ok(next.run(req).await)
}

/// Sliding-window in-process rate limiter (`17-api-security-operations.md §3.3`, resolves **TD-003**).
#[derive(Clone, Default)]
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn check_and_record(
        &self,
        bucket_key: &str,
        max_requests: usize,
        window: Duration,
    ) -> Result<(), AppError> {
        let now = Instant::now();
        let mut guard = self.buckets.lock().await;
        let timestamps = guard.entry(bucket_key.to_string()).or_default();
        timestamps.retain(|&ts| now.duration_since(ts) < window);

        if timestamps.len() >= max_requests {
            return Err(AppError::RateLimited);
        }

        timestamps.push(now);
        Ok(())
    }

    /// 10 requests / minute / IP for login, passkey challenge, password, and recovery code endpoints.
    pub async fn check_auth_ip(&self, ip: &str) -> Result<(), AppError> {
        self.check_and_record(&format!("auth_ip:{ip}"), 10, Duration::from_secs(60))
            .await
    }

    /// 3 requests / 15 minutes / email for magic link requests.
    pub async fn check_magic_link_email(&self, email: &str) -> Result<(), AppError> {
        let normalized = email.trim().to_ascii_lowercase();
        self.check_and_record(
            &format!("magic_email:{normalized}"),
            3,
            Duration::from_secs(15 * 60),
        )
        .await
    }

    /// 60 requests / minute / IP for anonymous search.
    pub async fn check_anonymous_search_ip(&self, ip: &str) -> Result<(), AppError> {
        self.check_and_record(&format!("search_ip:{ip}"), 60, Duration::from_secs(60))
            .await
    }
}

pub fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn generate_random_token_base64url() -> String {
    let mut raw = [0u8; 32];
    raw[..16].copy_from_slice(Uuid::new_v4().as_bytes());
    raw[16..].copy_from_slice(Uuid::new_v4().as_bytes());
    URL_SAFE_NO_PAD.encode(raw)
}

/// Validates password strength (min 10 chars per `17 §3.5`) and hashes with PBKDF2-HMAC-SHA256.
pub fn hash_password(password: &str) -> Result<String, AppError> {
    if password.chars().count() < 10 {
        let mut fields = HashMap::new();
        fields.insert("password".to_string(), "password_too_short".to_string());
        return Err(AppError::ValidationFailed(
            "Password must be at least 10 characters long".to_string(),
            Some(fields),
        ));
    }

    let salt_bytes = *Uuid::new_v4().as_bytes();
    let salt_hex = hex::encode(salt_bytes);
    let derived = pbkdf2_hmac_sha256(password.as_bytes(), &salt_bytes, PBKDF2_ITERATIONS);
    let hash_hex = hex::encode(derived);

    Ok(format!(
        "$pbkdf2-sha256$i={PBKDF2_ITERATIONS}${salt_hex}${hash_hex}"
    ))
}

pub fn verify_password(password: &str, stored_hash: &str) -> bool {
    let parts: Vec<&str> = stored_hash.split('$').filter(|s| !s.is_empty()).collect();
    if parts.len() != 4 || parts[0] != "pbkdf2-sha256" {
        return false;
    }

    let iterations = parts[1]
        .strip_prefix("i=")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    if iterations == 0 || iterations > 1_000_000 {
        return false;
    }

    let Ok(salt_bytes) = hex::decode(parts[2]) else {
        return false;
    };
    let Ok(expected_bytes) = hex::decode(parts[3]) else {
        return false;
    };

    let actual_bytes = pbkdf2_hmac_sha256(password.as_bytes(), &salt_bytes, iterations);
    constant_time_eq(&actual_bytes, &expected_bytes)
}

fn pbkdf2_hmac_sha256(password: &[u8], salt: &[u8], iterations: u32) -> [u8; 32] {
    // Standard single-block PBKDF2-HMAC-SHA256 (block index 1 = 0x00000001)
    let mut mac = HmacSha256::new_from_slice(password).expect("HMAC accepts any key length");
    mac.update(salt);
    mac.update(&1u32.to_be_bytes());
    let mut u = mac.finalize().into_bytes();
    let mut out = [0u8; 32];
    out.copy_from_slice(&u);

    for _ in 1..iterations {
        let mut step_mac =
            HmacSha256::new_from_slice(password).expect("HMAC accepts any key length");
        step_mac.update(&u);
        u = step_mac.finalize().into_bytes();
        for (o, b) in out.iter_mut().zip(u.iter()) {
            *o ^= *b;
        }
    }

    out
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (&x, &y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

pub fn normalize_recovery_code(raw: &str) -> String {
    raw.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

pub fn hash_recovery_code(raw: &str) -> String {
    let normalized = normalize_recovery_code(raw);
    sha256_hex(&format!("recovery:{normalized}"))
}

/// Generates 10 human-readable single-use recovery codes (`ABCD-EFGH-JKLM`) and their SHA-256 hashes.
pub fn generate_recovery_codes() -> Vec<(String, String)> {
    let mut codes = Vec::with_capacity(10);
    for _ in 0..10 {
        let bytes = Uuid::new_v4().into_bytes();
        let mut chars = String::with_capacity(14);
        for (i, &b) in bytes.iter().enumerate().take(12) {
            if i == 4 || i == 8 {
                chars.push('-');
            }
            let idx = (b as usize) % RECOVERY_ALPHABET.len();
            chars.push(RECOVERY_ALPHABET[idx] as char);
        }
        let hash = hash_recovery_code(&chars);
        codes.push((chars, hash));
    }
    codes
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_extract_sanitized_ip() {
        let mut headers = HeaderMap::new();

        // 1. Fallback default
        assert_eq!(extract_sanitized_ip(&headers, None), "127.0.0.1");

        // 2. Standard IPv4
        headers.insert("x-forwarded-for", HeaderValue::from_static("192.168.1.100"));
        assert_eq!(extract_sanitized_ip(&headers, None), "192.168.1.100");

        // 3. IPv4 with port
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("192.168.1.100:8080"),
        );
        assert_eq!(extract_sanitized_ip(&headers, None), "192.168.1.100");

        // 4. Multi-hop XFF: first hop is selected
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("10.0.0.1, 192.168.1.50, 172.16.0.1"),
        );
        assert_eq!(extract_sanitized_ip(&headers, None), "10.0.0.1");

        // 5. Standard IPv6
        headers.insert("x-forwarded-for", HeaderValue::from_static("2001:db8::1"));
        assert_eq!(extract_sanitized_ip(&headers, None), "2001:db8::1");

        // 6. IPv6 in brackets
        headers.insert("x-forwarded-for", HeaderValue::from_static("[2001:db8::1]"));
        assert_eq!(extract_sanitized_ip(&headers, None), "2001:db8::1");

        // 7. IPv6 in brackets with port
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("[2001:db8::1]:8443"),
        );
        assert_eq!(extract_sanitized_ip(&headers, None), "2001:db8::1");

        // 8. Malformed / injection in X-Forwarded-For falls back to X-Real-IP
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("127.0.0.1; DROP TABLE users"),
        );
        headers.insert("x-real-ip", HeaderValue::from_static("172.16.5.5"));
        assert_eq!(extract_sanitized_ip(&headers, None), "172.16.5.5");

        // 9. Malformed / injection in both headers falls back to 127.0.0.1
        headers.insert(
            "x-real-ip",
            HeaderValue::from_static("<script>alert(1)</script>"),
        );
        assert_eq!(extract_sanitized_ip(&headers, None), "127.0.0.1");

        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("999.999.999.999"),
        );
        headers.insert("x-real-ip", HeaderValue::from_static("invalid_ip"));
        assert_eq!(extract_sanitized_ip(&headers, None), "127.0.0.1");
    }

    #[test]
    fn test_extract_sanitized_user_agent() {
        let mut headers = HeaderMap::new();

        // Missing UA
        assert_eq!(extract_sanitized_user_agent(&headers), None);

        // Standard UA
        headers.insert(
            header::USER_AGENT,
            HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64)"),
        );
        assert_eq!(
            extract_sanitized_user_agent(&headers),
            Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64)".to_string())
        );

        // Control characters stripped (e.g. tabs)
        headers.insert(
            header::USER_AGENT,
            HeaderValue::from_static("Browser\tInjected-Header: value\tSafe"),
        );
        assert_eq!(
            extract_sanitized_user_agent(&headers),
            Some("BrowserInjected-Header: valueSafe".to_string())
        );

        // Long UA capped at 512 chars
        let long_ua = "a".repeat(1000);
        headers.insert(header::USER_AGENT, HeaderValue::from_str(&long_ua).unwrap());
        let sanitized = extract_sanitized_user_agent(&headers).unwrap();
        assert_eq!(sanitized.len(), 512);

        // Whitespace only
        headers.insert(header::USER_AGENT, HeaderValue::from_static("    "));
        assert_eq!(extract_sanitized_user_agent(&headers), None);
    }

    #[test]
    fn test_extract_sanitized_host_port() {
        let mut headers = HeaderMap::new();

        // 1. Fallback to localhost
        assert_eq!(
            extract_sanitized_host_port(&headers, None),
            ("localhost".to_string(), None)
        );

        // 2. Standard Host header
        headers.insert(
            header::HOST,
            HeaderValue::from_static("genealogy.example.com"),
        );
        assert_eq!(
            extract_sanitized_host_port(&headers, None),
            ("genealogy.example.com".to_string(), None)
        );

        // 3. Host with port
        headers.insert(
            header::HOST,
            HeaderValue::from_static("genealogy.example.com:8080"),
        );
        assert_eq!(
            extract_sanitized_host_port(&headers, None),
            ("genealogy.example.com".to_string(), Some(8080))
        );

        // 4. Uppercase Host is lowercased
        headers.insert(
            header::HOST,
            HeaderValue::from_static("GENEALOGY.EXAMPLE.COM:443"),
        );
        assert_eq!(
            extract_sanitized_host_port(&headers, None),
            ("genealogy.example.com".to_string(), Some(443))
        );

        // 5. X-Forwarded-Host takes precedence over Host
        headers.insert(
            "x-forwarded-host",
            HeaderValue::from_static("tree.public.org:9000, internal-proxy:8080"),
        );
        assert_eq!(
            extract_sanitized_host_port(&headers, None),
            ("tree.public.org".to_string(), Some(9000))
        );

        // 6. IPv6 in brackets
        headers.insert("x-forwarded-host", HeaderValue::from_static("[::1]:3000"));
        assert_eq!(
            extract_sanitized_host_port(&headers, None),
            ("::1".to_string(), Some(3000))
        );

        // 7. Control characters in host stripped
        headers.insert(
            "x-forwarded-host",
            HeaderValue::from_static("evil.com\t:80"),
        );
        assert_eq!(
            extract_sanitized_host_port(&headers, None),
            ("evil.com".to_string(), Some(80))
        );
    }

    #[test]
    fn test_detect_is_https() {
        let mut headers = HeaderMap::new();

        // 1. None -> false
        assert!(!detect_is_https(&headers, None));

        // 2. X-Forwarded-Proto: https
        headers.insert("x-forwarded-proto", HeaderValue::from_static("https"));
        assert!(detect_is_https(&headers, None));

        // 3. X-Forwarded-Proto: http
        headers.insert("x-forwarded-proto", HeaderValue::from_static("http"));
        assert!(!detect_is_https(&headers, None));

        // 4. X-Forwarded-Proto comma-separated
        headers.insert("x-forwarded-proto", HeaderValue::from_static("https, http"));
        assert!(detect_is_https(&headers, None));

        headers.remove("x-forwarded-proto");

        // 5. X-Forwarded-Ssl: on
        headers.insert("x-forwarded-ssl", HeaderValue::from_static("on"));
        assert!(detect_is_https(&headers, None));

        headers.insert("x-forwarded-ssl", HeaderValue::from_static("off"));
        assert!(!detect_is_https(&headers, None));

        headers.remove("x-forwarded-ssl");

        // 6. URI scheme
        let uri: Uri = "https://genealogy.example.com/api".parse().unwrap();
        assert!(detect_is_https(&headers, Some(&uri)));

        let uri_http: Uri = "http://genealogy.example.com/api".parse().unwrap();
        assert!(!detect_is_https(&headers, Some(&uri_http)));
    }

    #[test]
    fn test_compute_effective_rp_id() {
        // Explicit configured RP ID takes priority if non-local
        assert_eq!(
            compute_effective_rp_id("genealogy.company.org", "proxy.internal"),
            "genealogy.company.org"
        );

        // When configured RP ID is localhost, matches incoming host
        assert_eq!(
            compute_effective_rp_id("localhost", "familytree.levshitsvv.pro"),
            "familytree.levshitsvv.pro"
        );

        // When configured RP ID is 127.0.0.1, matches incoming host
        assert_eq!(
            compute_effective_rp_id("127.0.0.1", "familytree.levshitsvv.pro"),
            "familytree.levshitsvv.pro"
        );

        // Localhost config with localhost host stays localhost
        assert_eq!(
            compute_effective_rp_id("localhost", "localhost"),
            "localhost"
        );
        assert_eq!(
            compute_effective_rp_id("localhost", "127.0.0.1"),
            "localhost"
        );
        assert_eq!(compute_effective_rp_id("localhost", "::1"), "localhost");

        // Localhost host overrides configured remote RP ID for local development
        assert_eq!(
            compute_effective_rp_id("genealogy.company.org", "localhost"),
            "localhost"
        );
        assert_eq!(
            compute_effective_rp_id("genealogy.company.org", "127.0.0.1"),
            "localhost"
        );
        assert_eq!(
            compute_effective_rp_id("genealogy.company.org", "::1"),
            "localhost"
        );

        // Empty config falls back to host or localhost
        assert_eq!(
            compute_effective_rp_id("", "genealogy.example.com"),
            "genealogy.example.com"
        );
        assert_eq!(compute_effective_rp_id("", "localhost"), "localhost");
        assert_eq!(compute_effective_rp_id("", ""), "localhost");
    }

    #[test]
    fn test_is_same_origin() {
        let ctx = RequestContext {
            client_ip: "127.0.0.1".to_string(),
            user_agent: None,
            host: "genealogy.example.com".to_string(),
            port: None,
            is_https: true,
            effective_rp_id: "genealogy.example.com".to_string(),
            origin: None,
        };

        // Matching HTTPS origin without port
        assert!(is_same_origin("https://genealogy.example.com", &ctx));

        // Matching HTTPS origin with default port 443
        assert!(is_same_origin("https://genealogy.example.com:443", &ctx));

        // Scheme mismatch (HTTP instead of HTTPS)
        assert!(!is_same_origin("http://genealogy.example.com", &ctx));

        // Host mismatch
        assert!(!is_same_origin("https://attacker.example.com", &ctx));

        // Port mismatch
        assert!(!is_same_origin("https://genealogy.example.com:8443", &ctx));

        // Malformed origin
        assert!(!is_same_origin("not-a-valid-origin", &ctx));

        // Non-default port match
        let ctx_port = RequestContext {
            client_ip: "127.0.0.1".to_string(),
            user_agent: None,
            host: "localhost".to_string(),
            port: Some(3000),
            is_https: false,
            effective_rp_id: "localhost".to_string(),
            origin: None,
        };
        assert!(is_same_origin("http://localhost:3000", &ctx_port));
        assert!(!is_same_origin("http://localhost:8080", &ctx_port));
        assert!(!is_same_origin("https://localhost:3000", &ctx_port));
    }
}
