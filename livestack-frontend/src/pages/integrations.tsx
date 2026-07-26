import { useEffect, useState } from "react"
import { MessageCircle, Plug, Trash2 } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { api } from "@/lib/api"
import type { ChannelLink, ChannelLinksOutput } from "@/types/api"

const CHANNEL_LABELS: Record<string, string> = {
  telegram: "Telegram",
}

function maskChannelUserId(id: string) {
  if (id.length <= 4) return id
  return `••••${id.slice(-4)}`
}

export default function IntegrationsPage() {
  const [links, setLinks] = useState<ChannelLink[] | null>(null)
  const [loading, setLoading] = useState(true)
  const [pairingCode, setPairingCode] = useState("")
  const [linking, setLinking] = useState(false)
  const [removingId, setRemovingId] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  function loadLinks() {
    setLoading(true)
    api
      .get<ChannelLinksOutput>("/channels/links")
      .then((data) => setLinks(data.links))
      .catch(() => setError("Couldn't load linked channels."))
      .finally(() => setLoading(false))
  }

  useEffect(loadLinks, [])

  async function handleLink() {
    const code = pairingCode.trim()
    if (!code) return
    setLinking(true)
    setError(null)
    try {
      await api.post<ChannelLink>("/channels/link", { pairing_code: code })
      setPairingCode("")
      loadLinks()
    } catch {
      setError("That code didn't match — check it and try again.")
    } finally {
      setLinking(false)
    }
  }

  async function handleUnlink(id: string) {
    setRemovingId(id)
    setError(null)
    try {
      await api.delete(`/channels/links/${id}`)
      setLinks((prev) => prev?.filter((link) => link.id !== id) ?? prev)
    } catch {
      setError("Couldn't unlink that channel.")
    } finally {
      setRemovingId(null)
    }
  }

  return (
    <div className="space-y-6">
      <div className="border-b pb-6">
        <div className="mb-3 inline-flex items-center gap-2 text-xs font-medium text-muted-foreground">
          <Plug className="size-3.5 text-primary" />
          Connected channels
        </div>
        <h1 className="text-3xl font-semibold tracking-tight">Integrations</h1>
        <p className="mt-2 max-w-2xl text-sm leading-6 text-muted-foreground">
          Talk to your AI assistant from Telegram. Actions it takes there run immediately,
          the same way they would if you asked in the app.
        </p>
      </div>

      <section className="space-y-5">
        <div>
          <div className="flex items-center gap-2">
            <MessageCircle className="size-4 text-primary" />
            <h2 className="text-base font-semibold">Link Telegram</h2>
          </div>
          <p className="text-sm text-muted-foreground">
            Message the bot to get a pairing code, then enter it here to link that chat to your account.
          </p>
        </div>

        <div className="flex flex-col gap-2 sm:flex-row">
          <Input
            value={pairingCode}
            onChange={(e) => setPairingCode(e.target.value)}
            placeholder="Pairing code, e.g. 4F82A1"
          />
          <Button onClick={handleLink} disabled={linking || !pairingCode.trim()}>
            {linking ? "Linking..." : "Link"}
          </Button>
        </div>

        <div className="space-y-2 border-t pt-4">
          <h3 className="text-sm font-medium">Linked channels</h3>
          {loading && <p className="text-sm text-muted-foreground">Loading...</p>}
          {!loading && links && links.length === 0 && (
            <p className="text-sm text-muted-foreground">No channels linked yet.</p>
          )}
          {!loading &&
            links?.map((link) => (
              <div
                key={link.id}
                className="flex items-center justify-between gap-2 rounded-md border px-3 py-2 text-sm"
              >
                <div>
                  <span className="font-medium">{CHANNEL_LABELS[link.channel] ?? link.channel}</span>{" "}
                  <span className="text-muted-foreground">{maskChannelUserId(link.channel_user_id)}</span>
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => handleUnlink(link.id)}
                  disabled={removingId === link.id}
                >
                  <Trash2 className="size-4" />
                  {removingId === link.id ? "Removing..." : "Unlink"}
                </Button>
              </div>
            ))}
        </div>

        {error && <p className="text-sm text-destructive">{error}</p>}
      </section>
    </div>
  )
}
