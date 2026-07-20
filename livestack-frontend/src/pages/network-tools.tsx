import type { FormEvent } from "react"
import { useState } from "react"
import { Loader2, MapPin, Radar } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { api, ApiError } from "@/lib/api"
import type { NetworkHop } from "@/types/api"

export default function NetworkToolsPage() {
  const [target, setTarget] = useState("")
  const [hops, setHops] = useState<NetworkHop[]>([])
  const [tracing, setTracing] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function runTrace(e: FormEvent) {
    e.preventDefault()
    const trimmed = target.trim()
    if (!trimmed || tracing) return

    setTracing(true)
    setError(null)
    setHops([])

    try {
      for await (const hop of api.stream<NetworkHop>("/network-trace", { target: trimmed })) {
        setHops((prev) => [...prev, hop])
      }
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "Trace failed. Please try again.")
    } finally {
      setTracing(false)
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <div className="mb-1 inline-flex items-center gap-2 text-xs font-medium text-muted-foreground">
          <Radar className="size-3.5 text-primary" />
          Network Tools
        </div>
        <h1 className="text-3xl font-semibold tracking-tight">Trace a route</h1>
        <p className="mt-2 text-sm text-muted-foreground">
          Enter any URL or host to see the network path packets take to reach it, hop by hop.
        </p>
      </div>

      <form onSubmit={runTrace} className="flex items-center gap-2">
        <Input
          value={target}
          onChange={(e) => setTarget(e.target.value)}
          placeholder="example.com or https://example.com"
          disabled={tracing}
        />
        <Button type="submit" disabled={tracing || !target.trim()}>
          {tracing ? <Loader2 className="size-4 animate-spin" /> : <Radar className="size-4" />}
          {tracing ? "Tracing…" : "Run Trace"}
        </Button>
      </form>

      {error && <p className="text-sm text-destructive">{error}</p>}

      {hops.length > 0 && (
        <div className="overflow-hidden rounded-lg border">
          <table className="w-full text-sm">
            <thead className="border-b bg-muted/50 text-left text-xs font-medium uppercase tracking-wide text-muted-foreground">
              <tr>
                <th className="px-4 py-2">Hop</th>
                <th className="px-4 py-2">IP</th>
                <th className="px-4 py-2">Location</th>
                <th className="px-4 py-2">RTT</th>
              </tr>
            </thead>
            <tbody className="divide-y">
              {hops.map((hop) => (
                <tr key={hop.ttl}>
                  <td className="px-4 py-2 text-muted-foreground">{hop.ttl}</td>
                  <td className="px-4 py-2 font-mono">{hop.ip ?? "*"}</td>
                  <td className="px-4 py-2">
                    {hop.city || hop.country ? (
                      <span className="inline-flex items-center gap-1.5">
                        <MapPin className="size-3.5 text-muted-foreground" />
                        {[hop.city, hop.country].filter(Boolean).join(", ")}
                      </span>
                    ) : (
                      <span className="text-muted-foreground">—</span>
                    )}
                  </td>
                  <td className="px-4 py-2">
                    {hop.rtt_ms != null ? (
                      `${hop.rtt_ms.toFixed(1)} ms`
                    ) : (
                      <span className="text-muted-foreground">—</span>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {!tracing && !error && hops.length === 0 && (
        <div className="flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed py-16 text-center text-sm text-muted-foreground">
          <Radar className="size-6" />
          Run a trace to see the route.
        </div>
      )}
    </div>
  )
}
