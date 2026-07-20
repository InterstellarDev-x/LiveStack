import { useEffect, useState } from "react"
import { Link } from "react-router"
import { Activity, CheckCircle2, Siren } from "lucide-react"

import { IncidentList } from "@/components/incident-list"
import { api } from "@/lib/api"
import type { UserIncidentsOutput } from "@/types/api"

export default function IncidentsPage() {
  const [incidents, setIncidents] = useState<UserIncidentsOutput["incidents"]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    setLoading(true)
    setError(null)
    api
      .get<UserIncidentsOutput>("/incidents")
      .then((data) => setIncidents(data.incidents))
      .catch(() => setError("Couldn't load incidents."))
      .finally(() => setLoading(false))
  }, [])

  return (
    <div className="space-y-6">
      <div className="border-b pb-6">
        <div className="flex flex-col gap-5 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <div className="mb-3 inline-flex items-center gap-2 text-xs font-medium text-muted-foreground">
              <Siren className="size-3.5 text-primary" />
              Incident timeline
            </div>
            <h1 className="text-3xl font-semibold tracking-tight">Incidents</h1>
            <p className="mt-2 max-w-2xl text-sm leading-6 text-muted-foreground">
              Outages across all monitors, newest first, with duration and cause context for
              post-incident review.
            </p>
          </div>

          <div>
            <p className="text-2xl font-semibold">{loading ? "..." : incidents.length}</p>
            <p className="text-xs font-medium uppercase tracking-wider text-muted-foreground">
              Recorded
            </p>
          </div>
        </div>
      </div>

      {loading ? (
        <div className="py-6 text-sm text-muted-foreground">
          Loading incidents...
        </div>
      ) : error ? (
        <p className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {error}
        </p>
      ) : incidents.length === 0 ? (
        <div className="py-16 text-center">
          <CheckCircle2 className="mx-auto size-8 text-emerald-600" />
          <h2 className="mt-4 text-lg font-semibold">No incidents recorded</h2>
          <p className="mx-auto mt-2 max-w-md text-sm text-muted-foreground">
            Every check has come back healthy.{" "}
            <Link to="/monitors" className="font-medium text-primary underline underline-offset-4">
              View monitors
            </Link>
          </p>
        </div>
      ) : (
        <div>
          <div className="mb-2 flex items-center gap-2 py-2 text-xs font-medium uppercase tracking-wider text-muted-foreground">
            <Activity className="size-3.5" />
            Account-wide feed
          </div>
          <IncidentList
            incidents={incidents.map((incident) => ({ ...incident, label: incident.url }))}
            emptyMessage="No incidents yet."
          />
        </div>
      )}
    </div>
  )
}
