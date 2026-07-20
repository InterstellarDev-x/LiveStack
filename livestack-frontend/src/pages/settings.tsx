import { useEffect, useState } from "react"
import { BellRing, Mail, Save, Settings } from "lucide-react"

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
    return (
      <div className="py-6 text-sm text-muted-foreground">
        Loading settings...
      </div>
    )
  }

  if (!user) {
    return (
      <p className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
        {error ?? "Couldn't load account."}
      </p>
    )
  }

  return (
    <div className="space-y-6">
      <div className="border-b pb-6">
        <div className="mb-3 inline-flex items-center gap-2 text-xs font-medium text-muted-foreground">
          <Settings className="size-3.5 text-primary" />
          Workspace preferences
        </div>
        <h1 className="text-3xl font-semibold tracking-tight">Settings</h1>
        <p className="mt-2 max-w-2xl text-sm leading-6 text-muted-foreground">
          Configure where LiveStack sends status-change alerts for your production monitors.
        </p>
      </div>

      <section className="space-y-5">
        <div>
          <div className="flex items-center gap-2">
            <Mail className="size-4 text-primary" />
            <h2 className="text-base font-semibold">Notification email</h2>
          </div>
          <p className="text-sm text-muted-foreground">
            Used for status alerts across all of your monitors.
          </p>
        </div>

        <div className="flex flex-col gap-2 sm:flex-row">
          <Input
            type="email"
            value={emailDraft}
            onChange={(e) => setEmailDraft(e.target.value)}
            placeholder="you@example.com"
          />
          <Button onClick={handleSaveEmail} disabled={savingEmail}>
            <Save className="size-4" />
            {savingEmail ? "Saving..." : "Save"}
          </Button>
        </div>

        <div className="flex items-center justify-between gap-4 border-y py-4">
          <div>
            <div className="flex items-center gap-2">
              <BellRing className="size-4 text-primary" />
              <h2 className="text-sm font-semibold">Email alerts</h2>
            </div>
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
      </section>
    </div>
  )
}
