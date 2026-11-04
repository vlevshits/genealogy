#[allow(unused_imports)]
use super::*;

pub(crate) type HmacSha256 = Hmac<Sha256>;

pub(crate) const PBKDF2_ITERATIONS: u32 = 15_000;
pub(crate) const RECOVERY_ALPHABET: &[u8; 32] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";

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

pub(crate) fn parse_ip_candidate(raw: &str) -> Option<String> {
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
