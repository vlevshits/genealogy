use crate::error::AppError;
use crate::state::AppState;
use axum::{
    extract::{Request, State},
    http::Method,
    middleware::Next,
    response::Response,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

const PBKDF2_ITERATIONS: u32 = 15_000;
const RECOVERY_ALPHABET: &[u8; 32] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";

fn parse_host_port(authority: &str) -> Option<(&str, Option<u16>)> {
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

fn parse_origin_scheme_host_port(origin: &str) -> Option<(&str, &str, Option<u16>)> {
    let (scheme, rest) = origin.trim().split_once("://")?;
    if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
        return None;
    }
    let authority = rest.split('/').next().unwrap_or(rest);
    let (host, port) = parse_host_port(authority)?;
    Some((scheme, host, port))
}

fn is_same_origin_as_request(origin: &str, headers: &axum::http::HeaderMap) -> bool {
    let Some((origin_scheme, origin_host, origin_port)) = parse_origin_scheme_host_port(origin)
    else {
        return false;
    };

    let raw_target_host = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get("host"))
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(str::trim)
        .unwrap_or("");

    let Some((req_host, req_port)) = parse_host_port(raw_target_host) else {
        return false;
    };

    if !origin_host.eq_ignore_ascii_case(req_host) {
        return false;
    }

    if let Some(fwd_proto) = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if !origin_scheme.eq_ignore_ascii_case(fwd_proto) {
            return false;
        }
    }

    if let (Some(op), Some(rp)) = (origin_port, req_port) {
        if op != rp {
            return false;
        }
    }

    true
}

/// Enforces CSRF & Origin protection on all state-changing HTTP methods (`POST`, `PUT`, `PATCH`, `DELETE`)
/// as specified in `17-api-security-operations.md §3.2` (resolves **TD-002**):
/// 1. Requires `X-Requested-With: genealogy` header (case-insensitive value check).
/// 2. When `Origin` header is present, it must match one of the configured origins in `state.config.app_origin`,
///    or match the request's `X-Forwarded-Host` / `Host` (and `X-Forwarded-Proto` if present), or `http://localhost:*` in dev/test.
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

                let matches_request_host = is_same_origin_as_request(origin, req.headers());

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
