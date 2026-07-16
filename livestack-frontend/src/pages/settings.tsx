import { useEffect, useState } from "react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Switch } from "@/components/ui/switch"
import { api } from "@/lib/api"
import type { CurrentUser, UpdateEmailAlertsOutput, UpdateEmailOutput } from "@/types/api"

export default function SettingsPage() {
  const [user, setUser] = useState<CurrentUser | null>(null)
  const [emailDraft, setEmailDraft] = useState("")
  const [loading, setLoading] = useState(true)
  const [savingEmail, setSavingEmail] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    setLoading(true)
    api
      .get<CurrentUser>("/user/me")
      .then((data) => {
        setUser(data)
        setEmailDraft(data.email ?? "")
      })
      .catch(() => setError("Couldn't load account settings."))
      .finally(() => setLoading(false))
  }, [])

  async function handleSaveEmail() {
    setSavingEmail(true)
    setError(null)
    try {
      const result = await api.patch<UpdateEmailOutput>("/user/email", { email: emailDraft.trim() })
      setUser((prev) => (prev ? { ...prev, email: result.email } : prev))
    } catch {
      setError("Couldn't save email. Make sure it looks like a valid address.")
    } finally {
      setSavingEmail(false)
    }
  }

  async function handleToggleAlerts(enabled: boolean) {
    setUser((prev) => (prev ? { ...prev, email_alerts_enabled: enabled } : prev))
    try {
      const result = await api.patch<UpdateEmailAlertsOutput>("/user/notifications", { enabled })
      setUser((prev) => (prev ? { ...prev, email_alerts_enabled: result.email_alerts_enabled } : prev))
    } catch {
      setUser((prev) => (prev ? { ...prev, email_alerts_enabled: !enabled } : prev))
      setError("Couldn't update email alert preference.")
    }
  }

  if (loading) {
    return <p className="text-sm text-muted-foreground">Loading settings...</p>
  }

  if (!user) {
    return <p className="text-sm text-destructive">{error ?? "Couldn't load account."}</p>
  }

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">Settings</h1>

      <div className="space-y-4 rounded-lg border p-4">
        <div>
          <h2 className="text-sm font-medium">Notification email</h2>
          <p className="text-sm text-muted-foreground">
            Used for status alerts across all of your monitors.
          </p>
        </div>

        <div className="flex gap-2">
          <Input
            type="email"
            value={emailDraft}
            onChange={(e) => setEmailDraft(e.target.value)}
            placeholder="you@example.com"
          />
          <Button onClick={handleSaveEmail} disabled={savingEmail}>
            {savingEmail ? "Saving..." : "Save"}
          </Button>
        </div>

        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-sm font-medium">Email alerts</h2>
            <p className="text-sm text-muted-foreground">
              Send an email to the address above on every status change.
            </p>
          </div>
          <Switch
            checked={user.email_alerts_enabled}
            onCheckedChange={handleToggleAlerts}
            aria-label="Email alerts enabled"
          />
        </div>

        {error && <p className="text-sm text-destructive">{error}</p>}
      </div>
    </div>
  )
}
