import { useEffect, useState } from "react"
import { Link } from "react-router"

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
      <div>
        <h1 className="text-2xl font-semibold">Incidents</h1>
        <p className="text-sm text-muted-foreground">
          Outages across all your monitors, newest first.
        </p>
      </div>

      {loading ? (
        <p className="text-sm text-muted-foreground">Loading incidents...</p>
      ) : error ? (
        <p className="text-sm text-destructive">{error}</p>
      ) : incidents.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          No incidents yet — every check has come back healthy.{" "}
          <Link to="/monitors" className="underline underline-offset-4">
            View monitors
          </Link>
        </p>
      ) : (
        <IncidentList
          incidents={incidents.map((incident) => ({ ...incident, label: incident.url }))}
          emptyMessage="No incidents yet."
        />
      )}
    </div>
  )
}
