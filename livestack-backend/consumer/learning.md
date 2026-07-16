# Consumer Learnings

## 1. What actually happens on a request (beyond "DNS then IP then send")

```
1. DNS lookup     hostname -> IP address
2. TCP handshake   SYN -> SYN-ACK -> ACK              (open a connection to that IP:port)
3. TLS handshake    ClientHello -> ServerHello -> keys  (https only, before any HTTP bytes go out)
4. HTTP request sent  over the now-ready connection
5. Server "thinks"   backend does its work (DB queries, rendering, etc.)
6. First byte back   TTFB - the response starts arriving
7. Rest of body arrives  streams in until the response is complete
```

Getting the IP from DNS only tells you *where* to connect — steps 2 and 3 still have to
happen before a single byte of the actual HTTP request goes anywhere.

## 2. curl's timing fields are cumulative checkpoints, not durations

curl hands back timestamps measured from the start of the transfer, not the length of
each phase:

| curl field | Marks the moment... |
|---|---|
| `namelookup_time` | DNS resolution just finished |
| `connect_time` | TCP handshake just finished |
| `appconnect_time` | TLS handshake just finished (0 for plain `http://`) |
| `starttransfer_time` | first byte of the response arrived (TTFB) |
| `total_time` | the whole thing is done |

To get the duration of just one phase, subtract the previous checkpoint from the next
one. That's what `consumer/src/main.rs`'s `From<CurlTiming> for NewWebsiteTickTiming`
does:

```rust
dns_time_ms         = namelookup_time
connection_time_ms  = connect_time    - namelookup_time
tls_time_ms         = appconnect_time - connect_time              // 0 if no TLS happened
waiting_time_ms     = starttransfer_time - max(appconnect_time, connect_time)
data_transfer_time_ms = total_time - starttransfer_time
```

## 3. Correction: curl does NOT reset these timers per redirect hop

An earlier version of this note claimed `namelookup_time` / `connect_time` /
`appconnect_time` reset to 0 on each redirect hop and only describe the final one.
**That was wrong** — verified empirically, not just from docs:

```
$ curl -s -o /dev/null -L -w \
  "namelookup=%{time_namelookup} connect=%{time_connect} appconnect=%{time_appconnect} \
   starttransfer=%{time_starttransfer} total=%{time_total} redirect=%{time_redirect} \
   num_redirects=%{num_redirects}\n" "http://github.com"

namelookup=0.009 connect=0.105 appconnect=0.102 starttransfer=0.253 total=0.574 redirect=0.100 num_redirects=1
```

`namelookup` (9ms) is *smaller* than `redirect` (100ms). If `namelookup` only described
the final hop (after a 100ms redirect had already elapsed), it could never read 9ms —
that's not enough time to have happened *after* the redirect finished.

**What's actually true:** every curl timing field — `namelookup_time`, `connect_time`,
`appconnect_time`, `starttransfer_time`, `total_time`, `redirect_time` — is measured
from the *same single absolute start*, before hop 1. A field only advances when that
specific milestone happens again. In the `github.com` example: `http://github.com`
redirects to `https://github.com` (same host), so the DNS answer was reused — no second
lookup, `namelookup_time` never moves past hop 1's ~9ms. But the scheme change
(`http` → `https`) forces a brand new TCP + TLS connection, so `connect_time` /
`appconnect_time` *do* advance past the redirect, absorbing hop 1's entire ~100ms cost
along with it.

This makes "subtract `redirect_time` to isolate the final hop" **unsound in general** —
it goes negative here (`9 - 100 = -91`). Whether a field re-measures after a redirect
depends on whether that specific resource (DNS answer, TCP connection, TLS session) had
to be redone, which varies hop to hop and isn't something curl's easy-interface timing
fields expose cleanly. Getting a true "final hop only" breakdown would require following
redirects manually (one curl call per hop, `follow_location(false)`) — a bigger change,
not done here. `connection_time_ms` / `tls_time_ms` should be read as "cost of getting a
ready connection, including any redirects that forced a new one" rather than "just the
TLS handshake."

## 4. "Data transfer" used to be two unrelated things glued together — now split

Originally `data_transfer_time_ms` was computed as `total_time - max(appconnect_time,
connect_time)`, i.e. *everything* after the connection was ready. That bundled two very
different costs:
- the server thinking/processing time (ends at `starttransfer_time`, aka TTFB)
- the actual body download time (`total_time - starttransfer_time`)

A slow backend and a large response body looked identical in that one bucket. Fixed by
reading `starttransfer_time` and splitting it into two real columns:
- `waiting_time_ms` = TTFB = `starttransfer_time - max(appconnect_time, connect_time)`
- `data_transfer_time_ms` = `total_time - starttransfer_time` (download only, now accurate)

Verified against a real check (`https://github.com`):
`dns=41, connection=44, tls=70, waiting=0, data_transfer=282, total=437` — sums exactly
(`waiting=0` here isn't a bug, just millisecond-rounding: GitHub's CDN responded fast
enough that TTFB and the TLS handshake finishing landed in the same millisecond bucket).

## 5. Reference: what each stored `website_tick` field means

All five are *durations* (already deltas, not curl's raw cumulative checkpoints) —
this is what actually lands in the `website_tick` row per check:

| Column | Formula (from curl's cumulative fields) | What it really measures |
|---|---|---|
| `dns_time_ms` | `namelookup_time` | Time to resolve the hostname to an IP. Near-zero on a redirect hop that reuses an already-resolved host (§3). |
| `connection_time_ms` | `connect_time - namelookup_time` | Time to complete the TCP handshake. Includes any earlier redirect hop's full cost if that hop forced a new TCP connection (§3) — not purely "this handshake." |
| `tls_time_ms` | `appconnect_time - connect_time` (0 if `appconnect_time` is 0, i.e. plain `http://`) | Time to complete the TLS handshake once TCP was up. |
| `waiting_time_ms` | `starttransfer_time - max(appconnect_time, connect_time)` | Time between "connection ready" and "first byte received." **Not pure server processing** — it's the sum of (a) time to transmit the request, (b) the server actually doing work, and (c) network latency for the response to start coming back. curl can't separate these three. |
| `data_transfer_time_ms` | `total_time - starttransfer_time` | Time between "first byte received" and "whole response received." Correlates with body size ÷ throughput, but can also be inflated by TCP slow-start or a server that trickles the body out in small chunks even for a small payload. |
| `response_time_ms` | `total_time` (curl's own total, not a Rust-side wall-clock measurement — see the original migration notes) | Everything above, added together. Always equals the sum of the five phase columns. |

### Worked reconstruction from two real rows

Given only the stored deltas, you can rebuild curl's original cumulative checkpoints by
adding them back up — useful for sanity-checking a tick or explaining a specific number:

```
Row A:  dns=8   connection=147  tls=203  waiting=155  data_transfer=0    total=513
  namelookup    =                8
  connect       =            8+147 = 155
  appconnect    =          155+203 = 358
  starttransfer =          358+155 = 513
  total         =                    513   <- same millisecond as starttransfer:
                                             the entire body arrived alongside the
                                             first byte (tiny/empty response), so
                                             there was nothing left to "transfer".

Row B:  dns=2   connection=37   tls=50   waiting=313  data_transfer=163  total=565
  namelookup    =                2
  connect       =             2+37 = 39
  appconnect    =           39+50 = 89
  starttransfer =          89+313 = 402
  total         =         402+163 = 565

  Connection was ready fast (89ms), but 313ms passed with nothing coming back — real
  "Waiting" (request RTT + server work + response-start RTT). Then 163ms more to
  stream the rest of the body in once it started.
```
