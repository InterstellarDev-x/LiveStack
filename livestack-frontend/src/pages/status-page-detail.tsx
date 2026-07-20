import { useEffect, useState, type FormEvent } from "react"
import { Link, useParams } from "react-router"
import { ArrowLeft, ExternalLink, Globe, Plus, RadioTower, Trash2 } from "lucide-react"

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
        <Link to="/status-pages" className="inline-flex items-center gap-2 text-sm font-medium text-muted-foreground transition-colors hover:text-foreground">
          <ArrowLeft className="size-4" />
          Back to status pages
        </Link>
      </div>

      <div className="border-b pb-6">
        <div className="flex flex-col gap-5 sm:flex-row sm:items-start sm:justify-between">
          <div>
            <div className="mb-3 inline-flex items-center gap-2 text-xs font-medium text-muted-foreground">
              <Globe className="size-3.5 text-primary" />
              Public status page
            </div>
            <h1 className="text-3xl font-semibold tracking-tight">{page.title}</h1>
            <p className="mt-2 text-sm text-muted-foreground">/status/{page.slug}</p>
          </div>
          <Button variant="outline" render={<a href={`/status/${page.slug}`} target="_blank" rel="noreferrer" />}>
            <ExternalLink className="size-4" />
            View public page
          </Button>
        </div>
      </div>

      <section className="space-y-4">
        <div>
          <h2 className="text-base font-semibold">Published monitors</h2>
          <p className="text-sm text-muted-foreground">
            Choose which production services customers can see on this status page.
          </p>
        </div>

        {page.monitors.length === 0 ? (
          <div className="py-12 text-center">
            <RadioTower className="mx-auto size-8 text-primary" />
            <h3 className="mt-4 text-sm font-semibold">No monitors published yet</h3>
            <p className="mt-2 text-sm text-muted-foreground">
              Add a monitor below to make it visible on the public page.
            </p>
          </div>
        ) : (
          <ul className="divide-y divide-border border-y">
            {page.monitors.map((monitor) => (
              <li key={monitor.website_id} className="py-4">
                <div className="flex items-center justify-between gap-4">
                  <div className="flex min-w-0 flex-1 items-center gap-3">
                    <span className="flex size-9 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground">
                      <RadioTower className="size-4" />
                    </span>
                    <div className="min-w-0">
                      <p className="truncate text-sm font-semibold">{monitor.display_name}</p>
                      <p className="truncate text-xs text-muted-foreground">{monitor.url}</p>
                    </div>
                  </div>
                  <Button variant="ghost" size="icon" aria-label={`Remove ${monitor.display_name}`} onClick={() => handleRemoveMonitor(monitor.website_id)}>
                    <Trash2 className="size-4" />
                  </Button>
                </div>
              </li>
            ))}
          </ul>
        )}

        {availableWebsites.length > 0 && (
          <form onSubmit={handleAddMonitor} className="grid gap-2 lg:grid-cols-[1fr_0.72fr_auto]">
            <select
              className="h-9 min-w-0 rounded-md border border-input bg-transparent px-2.5 text-sm outline-none dark:bg-input/30"
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
              <Plus className="size-4" />
              {adding ? "Adding..." : "Add"}
            </Button>
          </form>
        )}

        {error && (
          <p className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
            {error}
          </p>
        )}
      </section>
    </div>
  )
}
