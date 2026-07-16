import { useEffect, useState, type FormEvent } from "react"
import { Link } from "react-router"

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
      <div>
        <h1 className="text-2xl font-semibold">Status Pages</h1>
        <p className="text-sm text-muted-foreground">
          Publish selected monitors to a page anyone can view.
        </p>
      </div>

      <form onSubmit={handleCreate} className="flex gap-2">
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
        <Button type="submit" disabled={creating}>
          {creating ? "Creating..." : "Create page"}
        </Button>
      </form>

      {error && <p className="text-sm text-destructive">{error}</p>}

      {loading ? (
        <p className="text-sm text-muted-foreground">Loading status pages...</p>
      ) : pages.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          No status pages yet. Create one above to get started.
        </p>
      ) : (
        <ul className="divide-y divide-border rounded-lg border">
          {pages.map((page) => (
            <li key={page.id} className="flex items-center justify-between gap-4 p-4">
              <Link to={`/status-pages/${page.id}`} className="min-w-0 flex-1">
                <p className="truncate text-sm font-medium">{page.title}</p>
                <p className="text-xs text-muted-foreground">/status/{page.slug}</p>
              </Link>
              <Button variant="ghost" size="sm" render={<a href={`/status/${page.slug}`} target="_blank" rel="noreferrer" />}>
                View
              </Button>
              <Button variant="ghost" size="sm" onClick={() => handleDelete(page.id)}>
                Delete
              </Button>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
