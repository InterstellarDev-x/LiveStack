use std::fmt;
use std::net::{IpAddr, Ipv6Addr, ToSocketAddrs};

use maxminddb::{geoip2, Reader};
use serde::Serialize;
use trippy_core::Builder;
use url::Url;

#[derive(Debug, Clone, Serialize)]
pub struct Hop {
    pub ttl: u8,
    pub ip: Option<String>,
    pub rtt_ms: Option<f64>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
}

#[derive(Debug)]
pub enum TraceError {
    InvalidTarget(String),
    PrivateTarget(String),
    ResolutionFailed(String),
    TraceFailed(String),
}

impl fmt::Display for TraceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTarget(msg) => write!(f, "invalid target: {msg}"),
            Self::PrivateTarget(host) => {
                write!(f, "target resolves to a private/reserved address: {host}")
            }
            Self::ResolutionFailed(msg) => write!(f, "could not resolve target: {msg}"),
            Self::TraceFailed(msg) => write!(f, "trace failed: {msg}"),
        }
    }
}

impl std::error::Error for TraceError {}

struct RawHop {
    ttl: u8,
    ip: Option<IpAddr>,
    rtt_ms: Option<f64>,
}

/// Run a single-round traceroute against `target` (a bare host or a URL) and
/// geolocate each responding hop using the given MaxMind GeoLite2-City reader.
pub async fn run_trace(geoip: &Reader<Vec<u8>>, target: &str) -> Result<Vec<Hop>, TraceError> {
    let host = extract_host(target)?;

    let raw_hops = tokio::task::spawn_blocking(move || resolve_and_trace(&host))
        .await
        .map_err(|err| TraceError::TraceFailed(err.to_string()))??;

    Ok(raw_hops
        .into_iter()
        .map(|hop| enrich_with_geoip(geoip, hop))
        .collect())
}

/// DNS resolution and the traceroute itself both block, so both run on the
/// same spawn_blocking thread rather than tying up a tokio worker.
fn resolve_and_trace(host: &str) -> Result<Vec<RawHop>, TraceError> {
    let ip = resolve_target(host)?;
    if is_private_or_reserved(ip) {
        return Err(TraceError::PrivateTarget(host.to_string()));
    }

    let tracer = Builder::new(ip)
        .max_rounds(Some(1))
        .build()
        .map_err(|err| TraceError::TraceFailed(err.to_string()))?;
    tracer
        .run()
        .map_err(|err| TraceError::TraceFailed(err.to_string()))?;

    let state = tracer.snapshot();
    Ok(state
        .hops()
        .iter()
        .map(|hop| RawHop {
            ttl: hop.ttl(),
            ip: hop.addrs().next().copied(),
            rtt_ms: hop.last_ms(),
        })
        .collect())
}

fn extract_host(input: &str) -> Result<String, TraceError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(TraceError::InvalidTarget("target is empty".to_string()));
    }
    let candidate = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("http://{trimmed}")
    };
    let url = Url::parse(&candidate).map_err(|err| TraceError::InvalidTarget(err.to_string()))?;
    url.host_str()
        .map(str::to_string)
        .ok_or_else(|| TraceError::InvalidTarget("target has no host".to_string()))
}

fn resolve_target(host: &str) -> Result<IpAddr, TraceError> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(ip);
    }
    (host, 0_u16)
        .to_socket_addrs()
        .map_err(|err| TraceError::ResolutionFailed(err.to_string()))?
        .next()
        .map(|addr| addr.ip())
        .ok_or_else(|| TraceError::ResolutionFailed(format!("no addresses found for {host}")))
}

/// Rejects loopback/private/link-local/reserved targets so this endpoint
/// can't be used to map the VM's own internal network.
fn is_private_or_reserved(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || v4.is_documentation()
        }
        IpAddr::V6(v6) => {
            v6.is_loopback() || v6.is_unspecified() || is_unique_local_v6(v6) || is_link_local_v6(v6)
        }
    }
}

fn is_unique_local_v6(ip: Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xfe00) == 0xfc00
}

fn is_link_local_v6(ip: Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xffc0) == 0xfe80
}

fn enrich_with_geoip(geoip: &Reader<Vec<u8>>, raw: RawHop) -> Hop {
    let geo = raw.ip.and_then(|ip| {
        geoip
            .lookup(ip)
            .ok()
            .and_then(|result| result.decode::<geoip2::City>().ok().flatten())
    });

    let (city, country, lat, lon) = match geo {
        Some(city_record) => (
            city_record.city.names.english.map(str::to_string),
            city_record.country.iso_code.map(str::to_string),
            city_record.location.latitude,
            city_record.location.longitude,
        ),
        None => (None, None, None, None),
    };

    Hop {
        ttl: raw.ttl,
        ip: raw.ip.map(|ip| ip.to_string()),
        rtt_ms: raw.rtt_ms,
        city,
        country,
        lat,
        lon,
    }
}
