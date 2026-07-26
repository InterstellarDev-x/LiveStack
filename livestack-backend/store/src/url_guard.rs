//! Validation for the two places a user hands us a URL that a *server*
//! process will later fetch: the monitor target (fetched by the check
//! consumer) and the notification webhook (POSTed to by the webhook worker).
//!
//! It lives here, rather than next to the HTTP handlers, because monitors can
//! also be created through the AI assistant's tools — two entry points that
//! must not disagree about what a valid target is.
//!
//! Neither was validated before, which made both a server-side request
//! forgery surface: pointing a monitor at `http://127.0.0.1:5432` or
//! `http://169.254.169.254/` turned the checker into an internal port
//! scanner (up/down and response time are reported back to the user), and a
//! webhook could aim the same requests at the cloud metadata endpoint.
//!
//! Scope, deliberately: this rejects private and reserved *literals* and the
//! obvious loopback names. It does not resolve hostnames, so it doesn't stop
//! a domain that resolves to a private address (including DNS rebinding) —
//! blocking that properly belongs at the egress layer, not in a parser.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use url::{Host, Url};

/// Long enough for any real endpoint; short enough to keep obviously
/// abusive input out of the database.
const MAX_URL_LEN: usize = 2048;

/// Hostnames that always mean "this machine".
const LOOPBACK_NAMES: &[&str] = &["localhost", "localhost.localdomain", "ip6-localhost"];

#[derive(Debug, PartialEq, Eq)]
pub enum UrlError {
    Empty,
    TooLong,
    Malformed,
    UnsupportedScheme,
    MissingHost,
    PrivateHost,
}

impl UrlError {
    /// Safe to show a user: it describes their input, nothing about ours.
    pub fn message(&self) -> &'static str {
        match self {
            Self::Empty => "url must not be empty",
            Self::TooLong => "url is too long",
            Self::Malformed => "url is not valid",
            Self::UnsupportedScheme => "url must use http:// or https://",
            Self::MissingHost => "url must include a host",
            Self::PrivateHost => "url must point at a public host",
        }
    }
}

/// Validates a monitor target and returns the canonical form to store.
///
/// A bare host (`example.com`) is accepted and normalized to `https://…`:
/// the UI has always let people type one, and the check consumer used to
/// paper over it at check time, which left the stored URL — the one shown in
/// the UI and in alert emails — different from the one actually fetched.
pub fn normalize_monitor_url(raw: &str) -> Result<String, UrlError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(UrlError::Empty);
    }
    if trimmed.len() > MAX_URL_LEN {
        return Err(UrlError::TooLong);
    }

    let candidate = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };

    Ok(parse_public_http_url(&candidate)?.to_string())
}

/// Validates a notification webhook URL. Unlike a monitor target this must
/// be written out in full — a webhook receiver is a specific endpoint, and
/// guessing a scheme for it would be papering over a typo.
pub fn validate_webhook_url(raw: &str) -> Result<(), UrlError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(UrlError::Empty);
    }
    if trimmed.len() > MAX_URL_LEN {
        return Err(UrlError::TooLong);
    }

    parse_public_http_url(trimmed).map(|_| ())
}

fn parse_public_http_url(candidate: &str) -> Result<Url, UrlError> {
    let url = Url::parse(candidate).map_err(|_| UrlError::Malformed)?;

    if !matches!(url.scheme(), "http" | "https") {
        return Err(UrlError::UnsupportedScheme);
    }

    match url.host() {
        None => Err(UrlError::MissingHost),
        Some(Host::Ipv4(ip)) if is_private_or_reserved(IpAddr::V4(ip)) => Err(UrlError::PrivateHost),
        Some(Host::Ipv6(ip)) if is_private_or_reserved(IpAddr::V6(ip)) => Err(UrlError::PrivateHost),
        Some(Host::Domain(domain)) if is_loopback_name(domain) => Err(UrlError::PrivateHost),
        Some(_) => Ok(url),
    }
}

fn is_loopback_name(domain: &str) -> bool {
    let domain = domain.trim_end_matches('.').to_ascii_lowercase();
    LOOPBACK_NAMES.contains(&domain.as_str()) || domain.ends_with(".localhost")
}

/// Mirrors `nettrace`'s target guard: anything that isn't routable on the
/// public internet, including the cloud metadata link-local range.
fn is_private_or_reserved(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || v4.is_documentation()
                || is_shared_v4(v4)
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || is_unique_local_v6(v6)
                || is_link_local_v6(v6)
                // ::ffff:127.0.0.1 and friends must not slip through.
                || v6.to_ipv4_mapped().is_some_and(|v4| is_private_or_reserved(IpAddr::V4(v4)))
        }
    }
}

/// 100.64.0.0/10, the carrier-grade NAT range (RFC 6598).
fn is_shared_v4(ip: Ipv4Addr) -> bool {
    ip.octets()[0] == 100 && (ip.octets()[1] & 0xc0) == 0x40
}

fn is_unique_local_v6(ip: Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xfe00) == 0xfc00
}

fn is_link_local_v6(ip: Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xffc0) == 0xfe80
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_host_is_normalized_to_https() {
        assert_eq!(
            normalize_monitor_url("example.com").unwrap(),
            "https://example.com/"
        );
    }

    #[test]
    fn explicit_scheme_and_path_are_preserved() {
        assert_eq!(
            normalize_monitor_url("http://example.com/health?deep=1").unwrap(),
            "http://example.com/health?deep=1"
        );
    }

    #[test]
    fn surrounding_whitespace_is_ignored() {
        assert_eq!(
            normalize_monitor_url("  https://example.com/  ").unwrap(),
            "https://example.com/"
        );
    }

    #[test]
    fn empty_and_malformed_targets_are_rejected() {
        assert_eq!(normalize_monitor_url("   "), Err(UrlError::Empty));
        assert_eq!(normalize_monitor_url("https://"), Err(UrlError::Malformed));
    }

    #[test]
    fn non_http_schemes_are_rejected() {
        assert_eq!(
            normalize_monitor_url("ftp://example.com"),
            Err(UrlError::UnsupportedScheme)
        );
        assert_eq!(
            normalize_monitor_url("file:///etc/passwd"),
            Err(UrlError::UnsupportedScheme)
        );
    }

    #[test]
    fn internal_targets_are_rejected() {
        for target in [
            "http://localhost:3000",
            "http://LOCALHOST/",
            "http://api.localhost/",
            "http://127.0.0.1/",
            "http://10.0.0.5/",
            "http://192.168.1.1/",
            "http://172.16.0.1/",
            "http://169.254.169.254/latest/meta-data/",
            "http://100.100.100.200/",
            "http://[::1]/",
            "http://[fd00::1]/",
            "http://[::ffff:127.0.0.1]/",
        ] {
            assert_eq!(
                normalize_monitor_url(target),
                Err(UrlError::PrivateHost),
                "expected {target} to be rejected"
            );
        }
    }

    #[test]
    fn public_addresses_are_allowed() {
        assert!(normalize_monitor_url("https://github.com/").is_ok());
        assert!(normalize_monitor_url("http://8.8.8.8/").is_ok());
        assert!(normalize_monitor_url("https://[2606:4700::1111]/").is_ok());
    }

    #[test]
    fn webhook_urls_need_an_explicit_scheme() {
        assert!(validate_webhook_url("https://hooks.example.com/x").is_ok());
        assert_eq!(
            validate_webhook_url("hooks.example.com/x"),
            Err(UrlError::Malformed)
        );
        assert_eq!(
            validate_webhook_url("http://169.254.169.254/"),
            Err(UrlError::PrivateHost)
        );
    }
}
