#[allow(unused_imports)]
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
