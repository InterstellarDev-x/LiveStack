import { useState, type FormEvent } from "react"
import { Link, useNavigate } from "react-router"
import { ArrowRight, BellRing, CheckCircle2, Globe, RadioTower, ShieldCheck } from "lucide-react"

import { BrandMark } from "@/components/brand-mark"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { api } from "@/lib/api"
import type { SignUpOutput } from "@/types/api"

const setupSteps = [
  "Add a website or API monitor",
  "Enable email or webhook alerts",
  "Publish a customer status page",
]

export default function SignupPage() {
  const [username, setUsername] = useState("")
  const [password, setPassword] = useState("")
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  const navigate = useNavigate()

  async function handleSubmit(event: FormEvent) {
    event.preventDefault()
    setError(null)
    setLoading(true)

    try {
      const data = await api.post<SignUpOutput>("/signup", { username, password })
      if (!data.success) {
        setError(data.message)
        return
      }
      navigate("/signin?registered=1", { replace: true })
    } catch {
      setError("Something went wrong. Please try again.")
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="min-h-svh bg-background text-foreground">
      <div className="mx-auto grid min-h-svh w-full max-w-6xl lg:grid-cols-[1.05fr_0.95fr]">
        <section className="hidden border-r bg-card p-6 lg:block">
          <div className="relative flex h-full overflow-hidden rounded-lg border bg-slate-950 p-6 text-white shadow-2xl shadow-primary/15">
            <div
              aria-hidden
              className="absolute inset-0 bg-[linear-gradient(to_right,rgba(255,255,255,0.08)_1px,transparent_1px),linear-gradient(to_bottom,rgba(255,255,255,0.08)_1px,transparent_1px)] bg-[size:38px_38px]"
            />
            <div aria-hidden className="absolute -left-28 top-12 size-72 rounded-full bg-primary/25 blur-3xl" />

            <div className="relative flex w-full flex-col justify-between">
              <Link to="/" className="flex w-fit items-center gap-2 text-sm font-semibold">
                <BrandMark />
                LiveStack
              </Link>

              <div className="space-y-6">
                <div>
                  <p className="text-xs font-semibold uppercase tracking-[0.18em] text-primary">
                    Production-ready from day one
                  </p>
                  <h2 className="mt-3 max-w-lg text-4xl font-semibold tracking-tight text-balance">
                    Start with the service your customers notice first.
                  </h2>
                </div>

                <div className="grid gap-3">
                  <div className="rounded-lg border border-white/10 bg-white/[0.07] p-4">
                    <div className="flex items-center justify-between gap-4">
                      <div className="flex items-center gap-3">
                        <RadioTower className="size-5 text-primary" />
                        <div>
                          <p className="text-sm font-semibold">checkout.service</p>
                          <p className="text-xs text-slate-400">Production monitor</p>
                        </div>
                      </div>
                      <span className="rounded-full bg-emerald-400/15 px-2.5 py-1 text-xs font-medium text-emerald-200">
                        Up
                      </span>
                    </div>
                  </div>

                  <div className="grid gap-3 sm:grid-cols-2">
                    <div className="rounded-lg border border-white/10 bg-white/[0.07] p-4">
                      <BellRing className="size-5 text-primary" />
                      <p className="mt-4 text-sm font-semibold">Alerts armed</p>
                      <p className="mt-1 text-xs text-slate-400">Email and webhook events</p>
                    </div>
                    <div className="rounded-lg border border-white/10 bg-white/[0.07] p-4">
                      <Globe className="size-5 text-primary" />
                      <p className="mt-4 text-sm font-semibold">Status page</p>
                      <p className="mt-1 text-xs text-slate-400">Public uptime history</p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>

        <section className="flex flex-col px-4 py-6 sm:px-8">
          <Link to="/" className="flex w-fit items-center gap-2 text-sm font-semibold lg:hidden">
            <BrandMark />
            LiveStack
          </Link>

          <div className="flex flex-1 items-center py-10">
            <div className="w-full max-w-md lg:ml-auto">
              <div className="mb-6 inline-flex items-center gap-2 rounded-md border bg-card px-3 py-1 text-xs font-medium text-muted-foreground">
                <ShieldCheck className="size-3.5 text-primary" />
                Create your reliability workspace
              </div>

              <div className="space-y-2">
                <h1 className="text-3xl font-semibold tracking-tight text-balance">
                  Bring your production checks into LiveStack.
                </h1>
                <p className="text-sm leading-6 text-muted-foreground">
                  Create an account, add your critical endpoints, and give your team one place to
                  see uptime, incidents, alerts, and public status.
                </p>
              </div>

              <div className="mt-6 grid gap-2">
                {setupSteps.map((step) => (
                  <p key={step} className="flex items-center gap-2 text-sm text-muted-foreground">
                    <CheckCircle2 className="size-4 text-emerald-600" />
                    {step}
                  </p>
                ))}
              </div>

              <form onSubmit={handleSubmit} className="mt-8 space-y-4">
                <div className="space-y-1.5">
                  <label htmlFor="username" className="text-sm font-medium">
                    Username
                  </label>
                  <Input
                    id="username"
                    autoComplete="username"
                    value={username}
                    onChange={(e) => setUsername(e.target.value)}
                    required
                  />
                </div>

                <div className="space-y-1.5">
                  <label htmlFor="password" className="text-sm font-medium">
                    Password
                  </label>
                  <Input
                    id="password"
                    type="password"
                    autoComplete="new-password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    required
                  />
                </div>

                {error && (
                  <p className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
                    {error}
                  </p>
                )}

                <Button type="submit" className="w-full" disabled={loading}>
                  {loading ? "Creating account..." : "Create workspace"}
                  {!loading && <ArrowRight className="size-4" />}
                </Button>
              </form>

              <p className="mt-6 text-sm text-muted-foreground">
                Already have an account?{" "}
                <Link to="/signin" className="font-medium text-primary underline underline-offset-4">
                  Sign in
                </Link>
              </p>
            </div>
          </div>
        </section>
      </div>
    </div>
  )
}
