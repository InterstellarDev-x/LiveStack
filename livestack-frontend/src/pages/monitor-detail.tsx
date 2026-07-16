import { useEffect, useState } from "react"
import { Link, useNavigate, useParams } from "react-router"

import { Button } from "@/components/ui/button"
import { ElapsedTime } from "@/components/elapsed-time"
import { IncidentList } from "@/components/incident-list"
import { ResponseTimeChart } from "@/components/response-time-chart"
import { StatusBadge } from "@/components/status-badge"
import { WebhookSettings } from "@/components/webhook-settings"
import { api } from "@/lib/api"
import { formatDateTime } from "@/lib/utils"
import type { WebsiteIncidentsOutput, WebsiteTicksOutput, WebsiteWithTick } from "@/types/api"

export default function MonitorDetailPage() {
  const { websiteId } = useParams()
  const navigate = useNavigate()

  const [site, setSite] = useState<WebsiteWithTick | null>(null)
  const [ticks, setTicks] = useState<WebsiteTicksOutput["ticks"]>([])
  const [incidents, setIncidents] = useState<WebsiteIncidentsOutput["incidents"]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!websiteId) return
    setLoading(true)
    setError(null)
    Promise.all([
      api.get<WebsiteWithTick>(`/website/${websiteId}`),
      api.get<WebsiteTicksOutput>(`/website/${websiteId}/ticks`),
      api.get<WebsiteIncidentsOutput>(`/website/${websiteId}/incidents`),
    ])
      .then(([siteData, ticksData, incidentsData]) => {
        setSite(siteData)
        setTicks(ticksData.ticks)
        setIncidents(incidentsData.incidents)
      })
      .catch(() => setError("Couldn't load this monitor."))
      .finally(() => setLoading(false))
  }, [websiteId])

  async function handleDelete() {
    if (!websiteId) return
    await api.delete(`/website/${websiteId}`)
    navigate("/monitors", { replace: true })
  }

  if (loading) {
    return <p className="text-sm text-muted-foreground">Loading monitor...</p>
  }

  if (error || !site) {
    return <p className="text-sm text-destructive">{error ?? "Monitor not found."}</p>
  }

  const tick = site.website_tick
  const status = tick?.status ?? "Unknown"

  return (
    <div className="space-y-6">
      <div>
        <Link to="/monitors" className="text-sm text-muted-foreground underline underline-offset-4">
          Back to monitors
        </Link>
      </div>

      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold break-all">{site.url}</h1>
          <p className="text-sm text-muted-foreground">
            Added {formatDateTime(site.time_added)}
          </p>
        </div>
        <Button variant="outline" onClick={handleDelete}>
          Delete monitor
        </Button>
      </div>

      <div className="rounded-lg border p-4">
        <StatusBadge status={status} />

        {tick ? (
          <dl className="mt-4 grid grid-cols-3 gap-4 text-sm">
            <div>
              <dt className="text-muted-foreground">Last checked</dt>
              <dd className="font-medium">
                <ElapsedTime since={tick.createdAt} />
              </dd>
            </div>
            <div>
              <dt className="text-muted-foreground">Response time</dt>
              <dd className="font-medium">{tick.response_time_ms}ms</dd>
            </div>
            <div>
              <dt className="text-muted-foreground">Region</dt>
              <dd className="font-medium">{tick.region_id}</dd>
            </div>
          </dl>
        ) : (
          <p className="mt-4 text-sm text-muted-foreground">
            No checks recorded yet.
          </p>
        )}
      </div>

      <div>
        <h2 className="mb-2 text-sm font-medium text-muted-foreground">Response time</h2>
        <ResponseTimeChart ticks={ticks} />
      </div>

      <div>
        <h2 className="mb-2 text-sm font-medium text-muted-foreground">Incidents</h2>
        <IncidentList
          incidents={incidents}
          emptyMessage="No incidents recorded — this monitor has never been confirmed down."
        />
      </div>

      <div>
        <h2 className="mb-2 text-sm font-medium text-muted-foreground">Notifications</h2>
        <WebhookSettings websiteId={site.id} />
      </div>

      <div>
        <h2 className="mb-2 text-sm font-medium text-muted-foreground">
          Last {ticks.length} checks
        </h2>
        {ticks.length === 0 ? (
          <p className="text-sm text-muted-foreground">No checks recorded yet.</p>
        ) : (
          <ul className="divide-y divide-border rounded-lg border">
            {ticks.map((t) => (
              <li key={t.id} className="flex items-center justify-between gap-4 px-4 py-2 text-sm">
                <div className="flex items-center gap-2">
                  <StatusBadge status={t.status} />
                  <span className="text-muted-foreground">{t.region_id}</span>
                </div>
                <span className="font-medium">{t.response_time_ms}ms</span>
                <span className="text-muted-foreground">{formatDateTime(t.createdAt)}</span>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  )
}
