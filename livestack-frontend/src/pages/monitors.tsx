import { useEffect, useState, type FormEvent } from "react"
import { Link } from "react-router"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { api } from "@/lib/api"
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
    try {
      await api.post<CreateWebsiteOutput>("/website", { url })
      setUrl("")
      await loadWebsites()
    } catch {
      setError("Couldn't add that monitor. Please try again.")
    } finally {
      setCreating(false)
    }
  }

  async function handleDelete(websiteId: string) {
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
      <div>
        <h1 className="text-2xl font-semibold">Monitors</h1>
        <p className="text-sm text-muted-foreground">
          Websites you're currently keeping an eye on.
        </p>
      </div>

      <form onSubmit={handleCreate} className="flex gap-2">
        <Input
          placeholder="https://example.com"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          required
        />
        <Button type="submit" disabled={creating}>
          {creating ? "Adding..." : "Add monitor"}
        </Button>
      </form>

      {error && <p className="text-sm text-destructive">{error}</p>}

      {loading ? (
        <p className="text-sm text-muted-foreground">Loading monitors...</p>
      ) : websites.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          No monitors yet. Add a URL above to get started.
        </p>
      ) : (
        <ul className="divide-y divide-border rounded-lg border">
          {websites.map((site) => (
            <li key={site.id} className="flex items-center justify-between gap-4 p-4">
              <Link to={`/monitors/${site.id}`} className="min-w-0 flex-1">
                <p className="truncate text-sm font-medium">{site.url}</p>
                <p className="text-xs text-muted-foreground">
                  Added {formatDateTime(site.time_added)}
                </p>
              </Link>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => handleDelete(site.id)}
              >
                Delete
              </Button>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
