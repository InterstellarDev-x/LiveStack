import { useEffect, useState } from "react"
import { Link, useNavigate, useParams } from "react-router"
import { Activity, ArrowLeft, Clock3, MapPin, Trash2, Zap } from "lucide-react"

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
        <Link to="/monitors" className="inline-flex items-center gap-2 text-sm font-medium text-muted-foreground transition-colors hover:text-foreground">
          <ArrowLeft className="size-4" />
          Back to monitors
        </Link>
      </div>

      <div className="border-b pb-6">
        <div className="flex flex-col gap-5 lg:flex-row lg:items-start lg:justify-between">
          <div className="min-w-0">
            <div className="mb-3 inline-flex items-center gap-2 text-xs font-medium text-muted-foreground">
              <Activity className="size-3.5 text-primary" />
              Monitor detail
            </div>
            <div className="flex flex-wrap items-center gap-3">
              <h1 className="break-all text-3xl font-semibold tracking-tight">{site.url}</h1>
              <StatusBadge status={status} />
            </div>
            <p className="mt-2 text-sm text-muted-foreground">
              Added {formatDateTime(site.time_added)}
            </p>
          </div>
          <Button variant="outline" onClick={handleDelete}>
            <Trash2 className="size-4" />
            Delete monitor
          </Button>
        </div>

        {tick ? (
          <dl className="mt-6 grid gap-3 sm:grid-cols-3">
            <div>
              <dt className="flex items-center gap-2 text-xs font-medium uppercase tracking-wider text-muted-foreground">
                <Clock3 className="size-3.5" />
                Last checked
              </dt>
              <dd className="mt-2 text-lg font-semibold">
                <ElapsedTime since={tick.createdAt} />
              </dd>
            </div>
            <div>
              <dt className="flex items-center gap-2 text-xs font-medium uppercase tracking-wider text-muted-foreground">
                <Zap className="size-3.5" />
                Response time
              </dt>
              <dd className="mt-2 text-lg font-semibold">{tick.response_time_ms}ms</dd>
            </div>
            <div>
              <dt className="flex items-center gap-2 text-xs font-medium uppercase tracking-wider text-muted-foreground">
                <MapPin className="size-3.5" />
                Region
              </dt>
              <dd className="mt-2 text-lg font-semibold">{tick.region_id}</dd>
            </div>
          </dl>
        ) : (
          <p className="mt-6 text-sm text-muted-foreground">
            No checks recorded yet.
          </p>
        )}
      </div>

      <section className="border-b pb-6">
        <div className="mb-4">
          <h2 className="text-base font-semibold">Response time</h2>
          <p className="text-sm text-muted-foreground">Recent latency samples for this monitor.</p>
        </div>
        <ResponseTimeChart ticks={ticks} />
      </section>

      <section className="border-b pb-6">
        <div className="mb-4">
          <h2 className="text-base font-semibold">Incidents</h2>
          <p className="text-sm text-muted-foreground">Confirmed outages and recovery history.</p>
        </div>
        <IncidentList
          incidents={incidents}
          emptyMessage="No incidents recorded — this monitor has never been confirmed down."
        />
      </section>

      <section className="border-b pb-6">
        <div className="mb-4">
          <h2 className="text-base font-semibold">Notifications</h2>
          <p className="text-sm text-muted-foreground">Send status changes to downstream tools.</p>
        </div>
        <WebhookSettings websiteId={site.id} />
      </section>

      <section>
        <div className="mb-4">
          <h2 className="text-base font-semibold">Last {ticks.length} checks</h2>
          <p className="text-sm text-muted-foreground">Raw check history for troubleshooting.</p>
        </div>
        {ticks.length === 0 ? (
          <p className="text-sm text-muted-foreground">No checks recorded yet.</p>
        ) : (
          <ul className="divide-y divide-border border-y">
            {ticks.map((t) => (
              <li key={t.id} className="grid gap-3 py-3 text-sm sm:grid-cols-[1fr_auto_auto] sm:items-center">
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
      </section>
    </div>
  )
}
