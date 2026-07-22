import { useState, type FormEvent } from "react"
import { Link, useNavigate } from "react-router"
import { ArrowRight, CheckCircle2, Gauge, LockKeyhole, RadioTower } from "lucide-react"

import { BrandMark } from "@/components/brand-mark"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { ApiError, api } from "@/lib/api"
import { useAuth } from "@/lib/auth"
import type { SignInOutput } from "@/types/api"

export default function SigninPage() {
  const [username, setUsername] = useState("")
  const [password, setPassword] = useState("")
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  const { login } = useAuth()
  const navigate = useNavigate()

  async function handleSubmit(event: FormEvent) {
    event.preventDefault()
    setError(null)
    setLoading(true)

    try {
      const data = await api.post<SignInOutput>("/signin", { username, password })
      login(data.token)
      navigate("/monitors", { replace: true })
    } catch (err) {
      setError(
        err instanceof ApiError && err.status === 401
          ? "Invalid username or password."
          : "Something went wrong. Please try again.",
      )
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="min-h-svh bg-background text-foreground">
      <div className="mx-auto grid min-h-svh w-full max-w-6xl lg:grid-cols-[0.95fr_1.05fr]">
        <section className="flex flex-col px-4 py-6 sm:px-8">
          <Link to="/" className="flex w-fit items-center gap-2 text-sm font-semibold">
            <BrandMark />
            LiveStack
          </Link>

          <div className="flex flex-1 items-center py-10">
            <div className="w-full max-w-md">
              <div className="mb-6 inline-flex items-center gap-2 rounded-md border bg-card px-3 py-1 text-xs font-medium text-muted-foreground">
                <LockKeyhole className="size-3.5 text-primary" />
                Secure operations console
              </div>

              <div className="space-y-2">
                <h1 className="text-3xl font-semibold tracking-tight text-balance">
                  Sign in to your production view.
                </h1>
                <p className="text-sm leading-6 text-muted-foreground">
                  Open your monitor list, incidents, status pages, alerts, and assistant from one
                  LiveStack workspace.
                </p>
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
                    autoComplete="current-password"
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
                  {loading ? "Signing in..." : "Sign in"}
                  {!loading && <ArrowRight className="size-4" />}
                </Button>
              </form>

              <p className="mt-6 text-sm text-muted-foreground">
                Don't have an account?{" "}
                <Link to="/signup" className="font-medium text-primary underline underline-offset-4">
                  Create a workspace
                </Link>
              </p>
            </div>
          </div>
        </section>

        <section className="hidden border-l bg-card p-6 lg:block">
          <div className="relative flex h-full overflow-hidden rounded-lg border bg-slate-950 p-6 text-white shadow-2xl shadow-primary/15">
            <div
              aria-hidden
              className="absolute inset-0 bg-[linear-gradient(to_right,rgba(255,255,255,0.08)_1px,transparent_1px),linear-gradient(to_bottom,rgba(255,255,255,0.08)_1px,transparent_1px)] bg-[size:38px_38px]"
            />
            <div aria-hidden className="absolute -right-28 top-16 size-72 rounded-full bg-primary/25 blur-3xl" />

            <div className="relative mt-auto w-full space-y-5">
              <div>
                <p className="text-xs font-semibold uppercase tracking-[0.18em] text-primary">
                  Current state
                </p>
                <h2 className="mt-3 max-w-md text-3xl font-semibold tracking-tight">
                  The fastest path back to what changed.
                </h2>
              </div>

              <div className="grid gap-3">
                <div className="rounded-lg border border-white/10 bg-white/[0.07] p-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <RadioTower className="size-5 text-primary" />
                      <div>
                        <p className="text-sm font-semibold">api.livestack.internal</p>
                        <p className="text-xs text-slate-400">Checked 12 seconds ago</p>
                      </div>
                    </div>
                    <span className="rounded-full bg-emerald-400/15 px-2.5 py-1 text-xs font-medium text-emerald-200">
                      Up
                    </span>
                  </div>
                </div>

                <div className="rounded-lg border border-white/10 bg-white/[0.07] p-4">
                  <div className="mb-4 flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <Gauge className="size-5 text-primary" />
                      <p className="text-sm font-semibold">Latency budget</p>
                    </div>
                    <span className="text-sm font-semibold">142ms</span>
                  </div>
                  <div className="h-2 overflow-hidden rounded-full bg-white/10">
                    <div className="h-full w-[68%] rounded-full bg-primary" />
                  </div>
                </div>

                <div className="rounded-lg border border-white/10 bg-white/[0.07] p-4">
                  <p className="mb-3 text-xs font-medium uppercase tracking-wider text-slate-400">
                    Response workflow
                  </p>
                  <div className="space-y-2 text-sm text-slate-200">
                    <p className="flex items-center gap-2">
                      <CheckCircle2 className="size-4 text-emerald-300" />
                      Incident history loaded
                    </p>
                    <p className="flex items-center gap-2">
                      <CheckCircle2 className="size-4 text-emerald-300" />
                      Status page ready
                    </p>
                    <p className="flex items-center gap-2">
                      <CheckCircle2 className="size-4 text-emerald-300" />
                      Alert channels armed
                    </p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>
  )
}
