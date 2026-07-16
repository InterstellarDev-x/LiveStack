import { useEffect, useState, type FormEvent } from "react"
import { Link, useParams } from "react-router"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { api } from "@/lib/api"
import type {
  StatusPageActionOutput,
  StatusPageDetail,
  StatusPageMonitor,
  Website,
  WebsitesByUserOutput,
} from "@/types/api"

export default function StatusPageDetailPage() {
  const { statusPageId } = useParams()

  const [page, setPage] = useState<StatusPageDetail | null>(null)
  const [websites, setWebsites] = useState<Website[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const [selectedWebsiteId, setSelectedWebsiteId] = useState("")
  const [displayName, setDisplayName] = useState("")
  const [adding, setAdding] = useState(false)

  async function loadPage() {
    if (!statusPageId) return
    setLoading(true)
    setError(null)
    try {
      const [pageData, websitesData] = await Promise.all([
        api.get<StatusPageDetail>(`/status-pages/${statusPageId}`),
        api.get<WebsitesByUserOutput>("/websites"),
      ])
      setPage(pageData)
      setWebsites(websitesData.websites)
    } catch {
      setError("Couldn't load this status page.")
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadPage()
  }, [statusPageId])

  const availableWebsites = websites.filter(
    (site) => !page?.monitors.some((monitor) => monitor.website_id === site.id)
  )

  async function handleAddMonitor(event: FormEvent) {
    event.preventDefault()
    if (!statusPageId || !selectedWebsiteId) return
    setAdding(true)
    setError(null)
    try {
      await api.post<StatusPageMonitor>(`/status-pages/${statusPageId}/monitors`, {
        website_id: selectedWebsiteId,
        display_name: displayName,
        sort_order: page?.monitors.length ?? 0,
      })
      setSelectedWebsiteId("")
      setDisplayName("")
      await loadPage()
    } catch {
      setError("Couldn't add that monitor. Please try again.")
    } finally {
      setAdding(false)
    }
  }

  async function handleRemoveMonitor(websiteId: string) {
    if (!statusPageId || !page) return
    setPage({ ...page, monitors: page.monitors.filter((m) => m.website_id !== websiteId) })
    try {
      await api.delete<StatusPageActionOutput>(
        `/status-pages/${statusPageId}/monitors/${websiteId}`
      )
    } catch {
      setError("Couldn't remove that monitor. Please try again.")
      loadPage()
    }
  }

  if (loading) {
    return <p className="text-sm text-muted-foreground">Loading status page...</p>
  }

  if (error && !page) {
    return <p className="text-sm text-destructive">{error}</p>
  }

  if (!page) {
    return <p className="text-sm text-destructive">Status page not found.</p>
  }

  return (
    <div className="space-y-6">
      <div>
        <Link to="/status-pages" className="text-sm text-muted-foreground underline underline-offset-4">
          Back to status pages
        </Link>
      </div>

      <div>
        <h1 className="text-2xl font-semibold">{page.title}</h1>
        <p className="text-sm text-muted-foreground">
          Public URL:{" "}
          <a href={`/status/${page.slug}`} target="_blank" rel="noreferrer" className="underline underline-offset-4">
            /status/{page.slug}
          </a>
        </p>
      </div>

      <div className="space-y-4 rounded-lg border p-4">
        <h2 className="text-sm font-medium">Monitors on this page</h2>

        {page.monitors.length === 0 ? (
          <p className="text-sm text-muted-foreground">No monitors published yet.</p>
        ) : (
          <ul className="divide-y divide-border rounded-lg border">
            {page.monitors.map((monitor) => (
              <li key={monitor.website_id} className="flex items-center justify-between gap-4 p-3">
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-medium">{monitor.display_name}</p>
                  <p className="truncate text-xs text-muted-foreground">{monitor.url}</p>
                </div>
                <Button variant="ghost" size="sm" onClick={() => handleRemoveMonitor(monitor.website_id)}>
                  Remove
                </Button>
              </li>
            ))}
          </ul>
        )}

        {availableWebsites.length > 0 && (
          <form onSubmit={handleAddMonitor} className="flex gap-2">
            <select
              className="h-8 min-w-0 flex-1 rounded-lg border border-input bg-transparent px-2.5 text-sm outline-none dark:bg-input/30"
              value={selectedWebsiteId}
              onChange={(e) => setSelectedWebsiteId(e.target.value)}
              required
            >
              <option value="" disabled>
                Select a monitor to publish
              </option>
              {availableWebsites.map((site) => (
                <option key={site.id} value={site.id}>
                  {site.url}
                </option>
              ))}
            </select>
            <Input
              placeholder="Display name, e.g. API"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              required
            />
            <Button type="submit" disabled={adding}>
              {adding ? "Adding..." : "Add"}
            </Button>
          </form>
        )}

        {error && <p className="text-sm text-destructive">{error}</p>}
      </div>
    </div>
  )
}
