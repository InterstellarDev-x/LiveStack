import { useEffect, useState } from "react"
import { useParams } from "react-router"

import { StatusBadge } from "@/components/status-badge"
import { api, ApiError } from "@/lib/api"
import { formatDateTime, formatDuration, toUtcDate } from "@/lib/utils"
import type { PublicStatusPage as PublicStatusPageData, PublicStatusPageIncident } from "@/types/api"

function formatUptime(value: number | null) {
  return value === null ? "—" : `${value.toFixed(2)}%`
}

function outageLength(incident: PublicStatusPageIncident) {
  if (!incident.resolved_at) return "ongoing"
  const seconds =
    (toUtcDate(incident.resolved_at).getTime() - toUtcDate(incident.started_at).getTime()) / 1000
  return formatDuration(seconds)
}

export default function PublicStatusPage() {
  const { slug } = useParams()

  const [page, setPage] = useState<PublicStatusPageData | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!slug) return
    setLoading(true)
    setError(null)
    api
      .get<PublicStatusPageData>(`/public/status-pages/${slug}`)
      .then(setPage)
      .catch((err) => {
        setError(err instanceof ApiError && err.status === 404 ? "Status page not found." : "Couldn't load this status page.")
      })
      .finally(() => setLoading(false))
  }, [slug])

  if (loading) {
    return (
      <div className="mx-auto max-w-2xl p-6">
        <p className="text-sm text-muted-foreground">Loading status...</p>
      </div>
    )
  }

  if (error || !page) {
    return (
      <div className="mx-auto max-w-2xl p-6">
        <p className="text-sm text-destructive">{error ?? "Status page not found."}</p>
      </div>
    )
  }

  const allUp = page.monitors.every((m) => m.status === "Up")
  const openIncidents = page.incidents.filter((incident) => incident.resolved_at === null)
  const pastIncidents = page.incidents.filter((incident) => incident.resolved_at !== null)

  return (
    <div className="mx-auto max-w-2xl space-y-6 p-6">
      <div>
        <h1 className="text-2xl font-semibold">{page.title}</h1>
        <p className="text-sm text-muted-foreground">
          {page.monitors.length === 0
            ? "No monitors published yet."
            : allUp && openIncidents.length === 0
              ? "All systems operational"
              : "Some systems are experiencing issues"}
        </p>
      </div>

      {openIncidents.length > 0 && (
        <div className="rounded-lg border border-destructive/50 bg-destructive/10 p-4">
          <h2 className="text-sm font-semibold text-destructive">Active incident</h2>
          <ul className="mt-2 space-y-1 text-sm">
            {/* Display names aren't unique — nothing stops two monitors on a
                page sharing one — so the position in the list is the only
                key that's actually distinct. */}
            {openIncidents.map((incident, i) => (
              <li key={i}>
                <span className="font-medium">{incident.display_name}</span>
                <span className="text-muted-foreground">
                  {" "}
                  — down since {formatDateTime(incident.started_at)}
                  {incident.cause ? ` (${incident.cause})` : ""}
                </span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {page.monitors.length > 0 && (
        <ul className="divide-y divide-border rounded-lg border">
          {page.monitors.map((monitor, i) => (
            <li key={i} className="flex items-center justify-between gap-4 p-4">
              <div className="flex items-center gap-2">
                <StatusBadge status={monitor.status} />
                <span className="text-sm font-medium">{monitor.display_name}</span>
              </div>
              <dl className="flex gap-4 text-xs text-muted-foreground">
                <div>
                  <dt className="inline">24h: </dt>
                  <dd className="inline font-medium text-foreground">{formatUptime(monitor.uptime_24h)}</dd>
                </div>
                <div>
                  <dt className="inline">7d: </dt>
                  <dd className="inline font-medium text-foreground">{formatUptime(monitor.uptime_7d)}</dd>
                </div>
                <div>
                  <dt className="inline">30d: </dt>
                  <dd className="inline font-medium text-foreground">{formatUptime(monitor.uptime_30d)}</dd>
                </div>
              </dl>
            </li>
          ))}
        </ul>
      )}

      {pastIncidents.length > 0 && (
        <div>
          <h2 className="mb-2 text-sm font-medium text-muted-foreground">
            Past incidents (last 30 days)
          </h2>
          <ul className="divide-y divide-border rounded-lg border">
            {pastIncidents.map((incident, i) => (
              <li
                key={i}
                className="flex flex-wrap items-center justify-between gap-x-4 gap-y-1 px-4 py-3 text-sm"
              >
                <div>
                  <span className="font-medium">{incident.display_name}</span>
                  {incident.cause && (
                    <span className="text-muted-foreground"> — {incident.cause}</span>
                  )}
                </div>
                <div className="flex items-center gap-4 text-muted-foreground">
                  <span>{formatDateTime(incident.started_at)}</span>
                  <span className="font-medium text-foreground">{outageLength(incident)}</span>
                </div>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  )
}
