import { Navigate, Link } from "react-router"
import { Activity, ArrowRight, Globe, Siren, Workflow } from "lucide-react"

import { Button } from "@/components/ui/button"
import { StatusBadge } from "@/components/status-badge"
import { useAuth } from "@/lib/auth"

const features = [
  {
    icon: Activity,
    title: "Uptime monitoring",
    description: "Track every website and API around the clock, with response time history at a glance.",
  },
  {
    icon: Siren,
    title: "Incident management",
    description: "Get notified the moment something goes down, and keep a clear record of every outage.",
  },
  {
    icon: Workflow,
    title: "Escalation policies",
    description: "Route alerts to the right person automatically so nothing slips through the cracks.",
  },
  {
    icon: Globe,
    title: "Public status pages",
    description: "Share live status with your customers and keep them informed during incidents.",
  },
]

const previewMonitors = [
  { name: "api.acme.com", status: "Up" as const, latency: "142 ms" },
  { name: "acme.com", status: "Up" as const, latency: "88 ms" },
  { name: "checkout.acme.com", status: "Down" as const, latency: "timed out" },
  { name: "status.acme.com", status: "Up" as const, latency: "104 ms" },
]

export default function LandingPage() {
  const { token } = useAuth()

  if (token) {
    return <Navigate to="/monitors" replace />
  }

  return (
    <div className="flex min-h-svh flex-col">
      <header className="sticky top-0 z-10 border-b bg-background/80 backdrop-blur">
        <div className="mx-auto flex h-14 w-full max-w-5xl items-center justify-between px-4">
          <span className="text-base font-semibold">LiveStack</span>
          <nav className="flex items-center gap-2">
            <Button variant="ghost" size="sm" render={<Link to="/signin" />}>
              Sign in
            </Button>
            <Button size="sm" render={<Link to="/signup" />}>
              Get started
            </Button>
          </nav>
        </div>
      </header>

      <main className="flex-1">
        <section className="relative overflow-hidden">
          <div
            aria-hidden
            className="pointer-events-none absolute left-1/2 top-0 -z-10 h-120 w-225 -translate-x-1/2 rounded-full bg-primary/20 blur-3xl"
          />
          <div
            aria-hidden
            className="pointer-events-none absolute -bottom-24 -left-24 -z-10 size-72 rounded-full bg-primary/10 blur-3xl"
          />

          <div className="mx-auto flex w-full max-w-5xl flex-col items-center gap-6 px-4 pt-28 pb-20 text-center">
            <div className="inline-flex items-center gap-1.5 rounded-full border bg-card px-3 py-1 text-xs font-medium text-muted-foreground">
              <span className="size-1.5 rounded-full bg-emerald-500" />
              All systems monitored in real time
            </div>

            <h1 className="max-w-2xl text-4xl font-semibold tracking-tight text-balance sm:text-5xl">
              Know the moment your site goes down
            </h1>
            <p className="max-w-xl text-lg text-muted-foreground text-balance">
              LiveStack monitors your websites and APIs, alerts your team the instant something breaks,
              and keeps your customers informed with a public status page.
            </p>
            <div className="flex items-center gap-3">
              <Button size="lg" render={<Link to="/signup" />}>
                Get started
                <ArrowRight />
              </Button>
              <Button variant="outline" size="lg" render={<Link to="/signin" />}>
                Sign in
              </Button>
            </div>

            <div className="mt-8 w-full max-w-md overflow-hidden rounded-[28px] border bg-card text-left shadow-[0_12px_32px_-4px_color-mix(in_oklch,var(--foreground)_18%,transparent)]">
              <div className="flex items-center gap-1.5 border-b bg-muted/40 px-4 py-2.5">
                <span className="size-2.5 rounded-full bg-destructive/40" />
                <span className="size-2.5 rounded-full bg-amber-500/40" />
                <span className="size-2.5 rounded-full bg-emerald-500/40" />
              </div>
              <ul className="divide-y divide-border">
                {previewMonitors.map((monitor) => (
                  <li key={monitor.name} className="flex items-center justify-between gap-4 px-4 py-3">
                    <span className="text-sm font-medium">{monitor.name}</span>
                    <div className="flex items-center gap-3">
                      <span className="text-xs text-muted-foreground">{monitor.latency}</span>
                      <StatusBadge status={monitor.status} />
                    </div>
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </section>

        <section className="border-t bg-muted/30">
          <div className="mx-auto w-full max-w-5xl px-4 py-20">
            <div className="mx-auto max-w-xl text-center">
              <h2 className="text-2xl font-semibold tracking-tight">Everything you need to stay on top of downtime</h2>
              <p className="mt-2 text-sm text-muted-foreground">
                From detection to resolution to customer communication, in one place.
              </p>
            </div>

            <div className="mt-12 grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
              {features.map((feature) => (
                <div
                  key={feature.title}
                  className="group rounded-2xl border bg-card p-5 shadow-[0_1px_2px_color-mix(in_oklch,var(--foreground)_10%,transparent)] transition-all hover:-translate-y-0.5 hover:border-primary/40 hover:shadow-[0_8px_20px_color-mix(in_oklch,var(--foreground)_12%,transparent)]"
                >
                  <div className="flex size-9 items-center justify-center rounded-full bg-primary/10 transition-colors group-hover:bg-primary/15">
                    <feature.icon className="size-4.5 text-primary" />
                  </div>
                  <h3 className="mt-3 text-sm font-semibold">{feature.title}</h3>
                  <p className="mt-1.5 text-sm text-muted-foreground">{feature.description}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        <section className="border-t">
          <div className="mx-auto w-full max-w-5xl px-4 py-20">
            <div className="flex flex-col items-center gap-4 rounded-4xl bg-primary/5 px-8 py-16 text-center">
              <h2 className="text-2xl font-semibold tracking-tight">Ready to stop finding out about outages from your customers?</h2>
              <Button size="lg" render={<Link to="/signup" />}>
                Get started for free
                <ArrowRight />
              </Button>
            </div>
          </div>
        </section>
      </main>

      <footer className="border-t py-6">
        <div className="mx-auto w-full max-w-5xl px-4 text-sm text-muted-foreground">
          © {new Date().getFullYear()} LiveStack
        </div>
      </footer>
    </div>
  )
}
