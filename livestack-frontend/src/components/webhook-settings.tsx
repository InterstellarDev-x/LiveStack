import { useEffect, useState } from "react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Switch } from "@/components/ui/switch"
import { api } from "@/lib/api"
import type { SetWebsiteWebhookOutput, WebsiteWebhookConfig } from "@/types/api"

function maskSecret(secret: string) {
  return `${"•".repeat(8)}${secret.slice(-4)}`
}

export function WebhookSettings({ websiteId }: { websiteId: string }) {
  const [config, setConfig] = useState<WebsiteWebhookConfig | null>(null)
  const [urlDraft, setUrlDraft] = useState("")
  const [enabledDraft, setEnabledDraft] = useState(false)
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [regenerating, setRegenerating] = useState(false)
  const [secretRevealed, setSecretRevealed] = useState(false)
  const [copied, setCopied] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    setLoading(true)
    setError(null)
    api
      .get<WebsiteWebhookConfig>(`/website/${websiteId}/webhook`)
      .then((data) => {
        setConfig(data)
        setUrlDraft(data.webhook_url ?? "")
        setEnabledDraft(data.webhook_enabled)
      })
      .catch(() => setError("Couldn't load webhook settings."))
      .finally(() => setLoading(false))
  }, [websiteId])

  async function handleSave() {
    setSaving(true)
    setError(null)
    try {
      const result = await api.put<SetWebsiteWebhookOutput>(`/website/${websiteId}/webhook`, {
        webhook_url: urlDraft.trim() === "" ? null : urlDraft.trim(),
        webhook_enabled: enabledDraft,
      })
      setConfig((prev) => ({
        webhook_url: result.webhook_url,
        webhook_enabled: result.webhook_enabled,
        webhook_secret: prev?.webhook_secret ?? result.webhook_secret ?? null,
      }))
    } catch {
      setError("Couldn't save webhook settings.")
    } finally {
      setSaving(false)
    }
  }

  async function handleRegenerate() {
    if (!window.confirm("Regenerate the signing secret? Any existing integration using the old secret will stop verifying.")) {
      return
    }
    setRegenerating(true)
    setError(null)
    try {
      const result = await api.post<WebsiteWebhookConfig>(
        `/website/${websiteId}/webhook/regenerate`
      )
      setConfig(result)
      setSecretRevealed(true)
    } catch {
      setError("Couldn't regenerate the secret.")
    } finally {
      setRegenerating(false)
    }
  }

  async function handleCopySecret() {
    if (!config?.webhook_secret) return
    await navigator.clipboard.writeText(config.webhook_secret)
    setCopied(true)
    setTimeout(() => setCopied(false), 1500)
  }

  if (loading) {
    return <p className="text-sm text-muted-foreground">Loading webhook settings...</p>
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-sm font-medium">Webhook</h2>
          <p className="text-sm text-muted-foreground">
            POST a signed payload to this URL whenever this monitor changes status.
          </p>
        </div>
        <Switch checked={enabledDraft} onCheckedChange={setEnabledDraft} aria-label="Webhook enabled" />
      </div>

      <div className="flex flex-col gap-2 sm:flex-row">
        <Input
          value={urlDraft}
          onChange={(e) => setUrlDraft(e.target.value)}
          placeholder="https://example.com/webhooks/livestack"
        />
        <Button onClick={handleSave} disabled={saving}>
          {saving ? "Saving..." : "Save"}
        </Button>
      </div>

      {config?.webhook_secret && (
        <div className="flex items-center justify-between gap-2 text-sm">
          <div>
            <span className="text-muted-foreground">Signing secret: </span>
            <code className="rounded bg-muted px-1.5 py-0.5">
              {secretRevealed ? config.webhook_secret : maskSecret(config.webhook_secret)}
            </code>
          </div>
          <div className="flex gap-1.5">
            <Button variant="ghost" size="sm" onClick={() => setSecretRevealed((v) => !v)}>
              {secretRevealed ? "Hide" : "Reveal"}
            </Button>
            <Button variant="ghost" size="sm" onClick={handleCopySecret}>
              {copied ? "Copied" : "Copy"}
            </Button>
            <Button variant="outline" size="sm" onClick={handleRegenerate} disabled={regenerating}>
              {regenerating ? "Regenerating..." : "Regenerate"}
            </Button>
          </div>
        </div>
      )}

      {error && <p className="text-sm text-destructive">{error}</p>}
    </div>
  )
}
