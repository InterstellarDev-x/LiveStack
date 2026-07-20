import { useEffect, useState, type FormEvent } from "react"
import { Link } from "react-router"
import { ExternalLink, Globe, Plus, Trash2 } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { api } from "@/lib/api"
import type { StatusPage, StatusPageActionOutput, StatusPagesOutput } from "@/types/api"

export default function StatusPagesPage() {
  const [pages, setPages] = useState<StatusPage[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const [slug, setSlug] = useState("")
  const [title, setTitle] = useState("")
  const [creating, setCreating] = useState(false)

  async function loadPages() {
    setLoading(true)
    setError(null)
    try {
      const data = await api.get<StatusPagesOutput>("/status-pages")
      setPages(data.pages)
    } catch {
      setError("Couldn't load status pages. Please try again.")
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadPages()
  }, [])

  async function handleCreate(event: FormEvent) {
    event.preventDefault()
    setCreating(true)
    setError(null)
    try {
      await api.post("/status-pages", { slug, title })
      setSlug("")
      setTitle("")
      await loadPages()
    } catch {
      setError("Couldn't create that status page. Slugs must be unique and use only lowercase letters, digits, and hyphens.")
    } finally {
      setCreating(false)
    }
  }

  async function handleDelete(statusPageId: string) {
    setPages((current) => current.filter((page) => page.id !== statusPageId))
    try {
      await api.delete<StatusPageActionOutput>(`/status-pages/${statusPageId}`)
    } catch {
      setError("Couldn't delete that status page. Please try again.")
      loadPages()
    }
  }

  return (
    <div className="space-y-6">
      <div className="border-b pb-6">
        <div className="flex flex-col gap-5 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <div className="mb-3 inline-flex items-center gap-2 text-xs font-medium text-muted-foreground">
              <Globe className="size-3.5 text-primary" />
              Customer communication
            </div>
            <h1 className="text-3xl font-semibold tracking-tight">Status Pages</h1>
            <p className="mt-2 max-w-2xl text-sm leading-6 text-muted-foreground">
              Publish selected monitors to a clear customer-facing page with uptime windows and
              incident history.
            </p>
          </div>

          <div>
            <p className="text-2xl font-semibold">{loading ? "..." : pages.length}</p>
            <p className="text-xs font-medium uppercase tracking-wider text-muted-foreground">
              Published
            </p>
          </div>
        </div>
      </div>

      <form onSubmit={handleCreate}>
        <div className="grid gap-2 lg:grid-cols-[1fr_0.7fr_auto]">
          <Input
            placeholder="Title, e.g. Acme Status"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            required
          />
          <Input
            placeholder="slug, e.g. acme"
            value={slug}
            onChange={(e) => setSlug(e.target.value.toLowerCase())}
            pattern="[a-z0-9-]+"
            required
          />
          <Button type="submit" disabled={creating} className="shrink-0">
            <Plus className="size-4" />
            {creating ? "Creating..." : "Create page"}
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
          Loading status pages...
        </div>
      ) : pages.length === 0 ? (
        <div className="py-16 text-center">
          <Globe className="mx-auto size-8 text-primary" />
          <h2 className="mt-4 text-lg font-semibold">No status pages yet</h2>
          <p className="mx-auto mt-2 max-w-md text-sm text-muted-foreground">
            Create a page above, then attach the monitors your customers should see.
          </p>
        </div>
      ) : (
        <ul className="divide-y divide-border border-y">
          {pages.map((page) => (
            <li key={page.id} className="py-4">
              <div className="flex items-center justify-between gap-4">
                <Link to={`/status-pages/${page.id}`} className="min-w-0 flex-1">
                  <div className="flex items-center gap-3">
                    <span className="flex size-9 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground">
                      <Globe className="size-4" />
                    </span>
                    <div className="min-w-0">
                      <p className="truncate text-sm font-semibold">{page.title}</p>
                      <p className="text-xs text-muted-foreground">/status/{page.slug}</p>
                    </div>
                  </div>
                </Link>
                <div className="flex items-center gap-1">
                  <Button variant="ghost" size="icon" aria-label={`View ${page.title}`} render={<a href={`/status/${page.slug}`} target="_blank" rel="noreferrer" />}>
                    <ExternalLink className="size-4" />
                  </Button>
                  <Button variant="ghost" size="icon" aria-label={`Delete ${page.title}`} onClick={() => handleDelete(page.id)}>
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
