# nettrace Learnings

## 1. What a traceroute fundamentally is

- Every IP packet has a TTL (time-to-live) field, decremented by one at each router hop.
- When a router decrements TTL to 0, it drops the packet and sends back an ICMP "Time Exceeded" message to the original sender, containing its own IP.
- A traceroute exploits this: send a probe with TTL=1, note who replies; send TTL=2, note who replies; increment TTL until a probe reaches the real destination.
- The result is a list of every router (hop) the packet passed through, in order, with the round-trip time to each.

## 2. The actual request path in this repo

1. Browser POSTs `{ target }` to `/network-trace` (`api/src/routes/network_trace.rs`), authenticated via the existing JWT middleware.
2. The handler calls `nettrace::run_trace(geoip, &input.target)` (`nettrace/src/lib.rs`).
3. `extract_host` normalizes the input — bare hostnames get an `http://` prefix slapped on so `url::Url::parse` can pull out just the host, whether the user typed `leetcode.com` or `https://leetcode.com/problems`.
4. `resolve_target` turns that hostname into an IP via normal DNS (`ToSocketAddrs`), or passes it through directly if it's already a raw IP.
5. `is_private_or_reserved` rejects loopback/private/link-local targets — otherwise this endpoint would let anyone probe the VM's own internal network (Postgres, Redis, etc. all sit on private/loopback addresses).
6. The actual trace runs via the `trippy-core` crate: `Builder::new(ip).max_rounds(Some(1)).build()` then `tracer.run()`. This is the part that needs a **raw socket** — it crafts the TTL-limited probe packets and listens for the raw ICMP replies directly, bypassing the normal OS socket API that regular apps use.
7. DNS + the trace itself are both *blocking* calls, so they're pushed onto a `spawn_blocking` thread instead of tying up an async tokio worker.
8. `tracer.snapshot()` returns each hop's TTL, responding IP (if any), and RTT.
9. Each responding hop gets geolocated via the local MaxMind `GeoLite2-City.mmdb` file (`enrich_with_geoip`) — city/country/lat/lon looked up straight from the IP, no external API call.
10. Results stream back to the browser one hop at a time over **SSE** (`Event::message(...).event_type("hop")`) rather than waiting for the whole trace to finish — same SSE mechanism the AI chat streaming uses.

## 3. Why it needed `CAP_NET_RAW` in production

- Raw sockets can craft arbitrary packets (source-address spoofing, sniffing traffic), so Linux restricts creating them to root or a process explicitly granted the `CAP_NET_RAW` capability.
- `api` runs as the unprivileged `azureuser` under pm2, so `trippy-core`'s raw socket creation failed with `Operation not permitted (os error 1): create new socket`.
- Fix: `sudo setcap cap_net_raw+ep target/release/api` grants just that one capability to just that one binary — no need to run the whole process as root.
- Gotcha: the capability is a **file attribute**, not a process attribute. Every deploy replaces the binary file, silently wiping it. Fixed by adding the `setcap` call into the CI deploy step itself, right after the new binary lands and before `pm2 reload`, so it's reapplied on every push.

## 4. Why most hops show "unknown"

- Not every router replies to the ICMP TTL-exceeded probe — many ISP backbone/cloud-edge routers are configured to ignore or heavily rate-limit ICMP, since it's diagnostic traffic, not real user traffic.
- A hop showing `unknown` just means that router never replied in time, not that anything is broken. Only the final destination (and sometimes the hop just before it) reliably answers, since it's the only one obligated to actually respond to complete the connection.
- This is normal for any traceroute to any real destination on the public internet, not a bug in this crate.

## 5. Why hitting the destination IP directly in a browser gives Cloudflare error 1003

- Big sites (leetcode.com included) sit behind Cloudflare, whose edge IPs are shared across many unrelated customer sites — same principle as nginx virtual hosting: one IP, many sites, disambiguated by the `Host` header (HTTP) or SNI (TLS).
- Pasting the bare IP into a browser sends a request with `Host: <ip>` instead of `Host: leetcode.com`, so Cloudflare's edge has no idea which of its thousands of hosted sites to route to.
- Error 1003 ("Direct IP Access Not Allowed") is Cloudflare deliberately refusing rather than guessing — an anti-abuse measure, not a network fault. The traceroute and the destination IP are both correct; only direct-IP browsing without the right `Host`/SNI fails.
