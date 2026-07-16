import { useMemo } from "react"
import { Area, AreaChart, ResponsiveContainer, Tooltip, XAxis } from "recharts"

import { formatDate, formatTime } from "@/lib/utils"
import type { WebsiteTick } from "@/types/api"

const REGION_FLAGS: Record<string, string> = {
  india: "🇮🇳",
}

// Fixed chronological order (name lookup happens first, data transfer last) —
// each phase always maps to the same categorical slot, never re-cycled.
const PHASES = [
  { key: "data_transfer_time_ms", label: "Data transfer", color: "var(--chart-6)" },
  { key: "waiting_time_ms", label: "Waiting (TTFB)", color: "var(--chart-5)" },
  { key: "tls_time_ms", label: "TLS handshake", color: "var(--chart-4)" },
  { key: "connection_time_ms", label: "Connection", color: "var(--chart-3)" },
  { key: "dns_time_ms", label: "Name lookup", color: "var(--chart-2)" },
] as const

function formatTimestamp(iso: string) {
  const time = formatTime(iso, {
    hour: "numeric",
    minute: "2-digit",
    second: "2-digit",
    timeZoneName: "short",
  })
  const day = formatDate(iso, { month: "short", day: "numeric" })
  return { time, day }
}

function ChartTooltip({
  active,
  payload,
}: {
  active?: boolean
  payload?: Array<{ payload: WebsiteTick }>
}) {
  if (!active || !payload?.length) return null

  const tick = payload[0].payload
  const { time, day } = formatTimestamp(tick.createdAt)

  return (
    <div className="min-w-56 rounded-lg border border-border bg-popover p-3 text-popover-foreground shadow-md">
      <p className="text-xs">
        <span className="font-medium">{time}</span>
        <span className="text-muted-foreground"> · {day}</span>
      </p>

      <div className="mt-2 flex items-start justify-between gap-2">
        <div>
          <p className="text-2xl leading-none font-semibold">{tick.response_time_ms}ms</p>
          <p className="text-xs text-muted-foreground">Total</p>
        </div>
        <span className="text-lg" aria-hidden>
          {REGION_FLAGS[tick.region_id] ?? "🌐"}
        </span>
      </div>

      <div className="mt-3 space-y-1.5 border-t border-border pt-2">
        {PHASES.map((phase) => (
          <div key={phase.key} className="flex items-center justify-between gap-4 text-xs">
            <span className="flex items-center gap-2 text-muted-foreground">
              <span
                className="h-2 w-2 rounded-full"
                style={{ backgroundColor: phase.color }}
                aria-hidden
              />
              {phase.label}
            </span>
            <span className="font-medium text-foreground">{tick[phase.key]}ms</span>
          </div>
        ))}
      </div>
    </div>
  )
}

export function ResponseTimeChart({ ticks }: { ticks: WebsiteTick[] }) {
  // Ticks arrive newest-first from the API; the chart reads left-to-right.
  const data = useMemo(() => [...ticks].reverse(), [ticks])

  if (data.length === 0) {
    return (
      <div className="flex h-48 items-center justify-center rounded-lg border text-sm text-muted-foreground">
        No checks recorded yet.
      </div>
    )
  }

  return (
    <div className="h-48 rounded-lg border p-2">
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart data={data} margin={{ top: 8, right: 8, left: 8, bottom: 0 }}>
          <defs>
            <linearGradient id="responseTimeFill" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor="var(--chart-1)" stopOpacity={0.25} />
              <stop offset="100%" stopColor="var(--chart-1)" stopOpacity={0} />
            </linearGradient>
          </defs>
          <XAxis
            dataKey="createdAt"
            tickFormatter={(value: string) => formatTime(value, { hour: "numeric", minute: "2-digit" })}
            tick={{ fontSize: 11, fill: "var(--muted-foreground)" }}
            axisLine={{ stroke: "var(--border)" }}
            tickLine={false}
            minTickGap={32}
          />
          <Tooltip content={<ChartTooltip />} cursor={{ stroke: "var(--border)", strokeWidth: 1 }} />
          <Area
            type="monotone"
            dataKey="response_time_ms"
            stroke="var(--chart-1)"
            strokeWidth={2}
            fill="url(#responseTimeFill)"
            dot={false}
            activeDot={{ r: 4, strokeWidth: 0 }}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  )
}
