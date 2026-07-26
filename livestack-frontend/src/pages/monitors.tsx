import { useEffect, useState, type FormEvent } from "react"
import { Link } from "react-router"
import { Activity, ArrowRight, Plus, RadioTower, Trash2 } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { ApiError, api } from "@/lib/api"
import { formatDateTime } from "@/lib/utils"
import type { CreateWebsiteOutput, Website, WebsitesByUserOutput } from "@/types/api"

export default function MonitorsPage() {
  const [websites, setWebsites] = useState<Website[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const [url, setUrl] = useState("")
  const [creating, setCreating] = useState(false)

  async function loadWebsites() {
    setLoading(true)
    setError(null)
    try {
      const data = await api.get<WebsitesByUserOutput>("/websites")
      setWebsites(data.websites)
    } catch {
      setError("Couldn't load monitors. Please try again.")
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadWebsites()
  }, [])

  async function handleCreate(event: FormEvent) {
    event.preventDefault()
    setCreating(true)
    setError(null)
    try {
      await api.post<CreateWebsiteOutput>("/website", { url })
      setUrl("")
      await loadWebsites()
    } catch (err) {
      // The server rejects unparseable URLs and internal hosts; say which,
      // rather than making the user guess at "please try again".
      setError(
        err instanceof ApiError && err.status === 400
          ? "That doesn't look like a monitorable URL. Use a public http:// or https:// address."
          : "Couldn't add that monitor. Please try again.",
      )
    } finally {
      setCreating(false)
    }
  }

  async function handleDelete(websiteId: string, siteUrl: string) {
    if (
      !window.confirm(`Delete the monitor for ${siteUrl}? Its check and incident history goes too.`)
    ) {
      return
    }

    setWebsites((current) => current.filter((site) => site.id !== websiteId))
    try {
      await api.delete(`/website/${websiteId}`)
    } catch {
      setError("Couldn't delete that monitor. Please try again.")
      loadWebsites()
    }
  }

  return (
    <div className="space-y-6">
      <div className="border-b pb-6">
        <div className="flex flex-col gap-5 lg:flex-row lg:items-end lg:justify-between">
          <div>
            <div className="mb-3 inline-flex items-center gap-2 text-xs font-medium text-muted-foreground">
              <RadioTower className="size-3.5 text-primary" />
              Production monitors
            </div>
            <h1 className="text-3xl font-semibold tracking-tight">Monitors</h1>
            <p className="mt-2 max-w-2xl text-sm leading-6 text-muted-foreground">
              Track the websites and APIs your organization depends on, then inspect response
              history, incidents, and notification settings per service.
            </p>
          </div>

          <div className="grid min-w-56 grid-cols-2 gap-3">
            <div>
              <p className="text-2xl font-semibold">{websites.length}</p>
              <p className="text-xs font-medium uppercase tracking-wider text-muted-foreground">
                Total
              </p>
            </div>
            <div>
              <p className="text-2xl font-semibold">{loading ? "..." : "Live"}</p>
              <p className="text-xs font-medium uppercase tracking-wider text-muted-foreground">
                Checks
              </p>
            </div>
          </div>
        </div>
      </div>

      <form onSubmit={handleCreate}>
        <div className="flex flex-col gap-2 sm:flex-row">
          <Input
            placeholder="https://api.company.com"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            required
          />
          <Button type="submit" disabled={creating} className="shrink-0">
            <Plus className="size-4" />
            {creating ? "Adding..." : "Add monitor"}
          </Button>
        </div>
      </form>

      {error && (
        <p className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {error}
        </p>
      )}

      {loading ? (
        <div className="py-6 text-sm text-muted-foreground">
          Loading monitors...
        </div>
      ) : websites.length === 0 ? (
        <div className="py-16 text-center">
          <Activity className="mx-auto size-8 text-primary" />
          <h2 className="mt-4 text-lg font-semibold">No monitors yet</h2>
          <p className="mx-auto mt-2 max-w-md text-sm text-muted-foreground">
            Add your first production URL above. LiveStack will start recording checks and incidents
            for that service.
          </p>
        </div>
      ) : (
        <ul className="divide-y divide-border border-y">
          {websites.map((site) => (
            <li key={site.id} className="py-4">
              <div className="flex items-center justify-between gap-4">
                <Link to={`/monitors/${site.id}`} className="min-w-0 flex-1">
                  <div className="flex items-center gap-3">
                    <span className="flex size-9 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground">
                      <Activity className="size-4" />
                    </span>
                    <div className="min-w-0">
                      <p className="truncate text-sm font-semibold">{site.url}</p>
                      <p className="text-xs text-muted-foreground">
                        Added {formatDateTime(site.time_added)}
                      </p>
                    </div>
                  </div>
                </Link>
                <div className="flex items-center gap-1">
                  <Button variant="ghost" size="icon" aria-label={`Open ${site.url}`} render={<Link to={`/monitors/${site.id}`} />}>
                    <ArrowRight className="size-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    aria-label={`Delete ${site.url}`}
                    onClick={() => handleDelete(site.id, site.url)}
                  >
                    <Trash2 className="size-4" />
                  </Button>
                </div>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
