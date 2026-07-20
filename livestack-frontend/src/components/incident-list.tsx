import { formatDateTime, formatDuration } from "@/lib/utils"

export interface IncidentRow {
  id: string
  /** Optional context label — the website URL on the account-wide feed. */
  label?: string
  started_at: string
  resolved_at: string | null
  cause: string
  duration_seconds: number | null
}

function OutageBadge({ ongoing }: { ongoing: boolean }) {
  return (
    <span
      className={`rounded-full px-2 py-0.5 text-xs font-medium ${
        ongoing
          ? "bg-destructive/10 text-destructive"
          : "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400"
      }`}
    >
      {ongoing ? "Ongoing" : "Resolved"}
    </span>
  )
}

export function IncidentList({
  incidents,
  emptyMessage,
}: {
  incidents: IncidentRow[]
  emptyMessage: string
}) {
  if (incidents.length === 0) {
    return <p className="text-sm text-muted-foreground">{emptyMessage}</p>
  }

  return (
    <ul className="divide-y divide-border border-y">
      {incidents.map((incident) => (
        <li key={incident.id} className="flex flex-wrap items-center justify-between gap-x-4 gap-y-1 py-3 text-sm">
          <div className="flex items-center gap-2">
            <OutageBadge ongoing={incident.resolved_at === null} />
            {incident.label && <span className="font-medium break-all">{incident.label}</span>}
            <span className="text-muted-foreground">{incident.cause || "unknown cause"}</span>
          </div>
          <div className="flex items-center gap-4 text-muted-foreground">
            <span>{formatDateTime(incident.started_at)}</span>
            <span className="font-medium text-foreground">
              {incident.duration_seconds !== null
                ? formatDuration(incident.duration_seconds)
                : "ongoing"}
            </span>
          </div>
        </li>
      ))}
    </ul>
  )
}
