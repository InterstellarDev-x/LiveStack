import { Navigate, Link } from "react-router"
import {
  ArrowRight,
  BellRing,
  Bot,
  CheckCircle2,
  Clock3,
  Gauge,
  Globe,
  RadioTower,
  ShieldCheck,
  Siren,
  Webhook,
} from "lucide-react"

import { BrandMark } from "@/components/brand-mark"
import { Button } from "@/components/ui/button"
import { StatusBadge } from "@/components/status-badge"
import { useAuth } from "@/lib/auth"

const operatingLoop = [
  {
    icon: RadioTower,
    title: "Detect the failure",
    description: "Continuously check production websites and APIs from your LiveStack monitors.",
  },
  {
    icon: Siren,
    title: "Open the incident trail",
    description: "Track outages, response time changes, causes, and resolution history in one place.",
  },
  {
    icon: BellRing,
    title: "Notify the right channels",
    description: "Send email alerts and monitor-level webhooks when status changes happen.",
  },
  {
    icon: Globe,
    title: "Keep users informed",
    description: "Publish selected services to customer-facing status pages with 24h, 7d, and 30d uptime.",
  },
]

const orgBenefits = [
  "Reduce customer-reported downtime by detecting failures before support tickets arrive.",
  "Give engineering, support, and leadership the same incident record instead of scattered messages.",
  "Use public status pages to lower inbound questions during known outages.",
]

const productionSignals = [
  { label: "API availability", value: "99.98%", icon: ShieldCheck },
  { label: "Checkout latency", value: "142ms", icon: Gauge },
  { label: "Open incidents", value: "1", icon: Siren },
]

const previewMonitors = [
  { name: "api.livestack.internal", status: "Up" as const, latency: "142 ms", region: "india" },
  { name: "checkout.service", status: "Down" as const, latency: "timed out", region: "asia-south1" },
  { name: "customer-status", status: "Up" as const, latency: "104 ms", region: "global" },
]

const assistantPrompts = [
  "Which services changed state today?",
  "Why was checkout slow yesterday?",
  "Create a monitor for the billing API",
]

export default function LandingPage() {
  const { token } = useAuth()

  if (token) {
    return <Navigate to="/monitors" replace />
  }

  return (
    <div className="min-h-svh bg-background text-foreground">
      <header className="sticky top-0 z-20 border-b bg-background/85 backdrop-blur">
        <div className="mx-auto flex h-16 w-full max-w-6xl items-center justify-between px-4">
          <Link to="/" className="flex items-center gap-2 text-sm font-semibold">
            <BrandMark />
            LiveStack
          </Link>
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

      <main>
        <section className="relative overflow-hidden border-b">
          <div
            aria-hidden
            className="absolute inset-0 bg-[linear-gradient(to_right,color-mix(in_oklch,var(--primary)_10%,transparent)_1px,transparent_1px),linear-gradient(to_bottom,color-mix(in_oklch,var(--primary)_10%,transparent)_1px,transparent_1px)] bg-[size:44px_44px]"
          />
          <div
            aria-hidden
            className="absolute left-1/2 top-0 h-96 w-[48rem] -translate-x-1/2 rounded-full bg-primary/12 blur-3xl"
          />
          <div className="relative mx-auto grid w-full max-w-6xl gap-10 px-4 py-14 lg:grid-cols-[1fr_0.95fr] lg:py-20">
            <div className="flex flex-col justify-center">
              <div className="mb-5 inline-flex w-fit items-center gap-2 rounded-md border bg-card/80 px-3 py-1 text-xs font-medium text-muted-foreground shadow-sm">
                <span className="size-2 rounded-full bg-emerald-500" />
                Production monitoring, incident response, and status communication
              </div>
              <h1 className="max-w-3xl text-5xl font-semibold leading-[0.98] tracking-tight text-balance sm:text-6xl lg:text-7xl">
                Run production with fewer blind spots.
              </h1>
              <p className="mt-6 max-w-2xl text-lg leading-8 text-muted-foreground text-balance">
                LiveStack watches your websites and APIs, records incidents, alerts your team, and
                gives customers a clear status page when something goes wrong.
              </p>
              <div className="mt-8 flex flex-col gap-3 sm:flex-row">
                <Button size="lg" render={<Link to="/signup" />}>
                  Start monitoring
                  <ArrowRight className="size-4" />
                </Button>
                <Button variant="outline" size="lg" className="bg-background/70" render={<Link to="/signin" />}>
                  Open console
                </Button>
              </div>

              <div className="mt-10 grid max-w-2xl gap-3 sm:grid-cols-3">
                {productionSignals.map((signal) => (
                  <div key={signal.label} className="rounded-lg border bg-card/80 p-4 shadow-sm">
                    <signal.icon className="mb-3 size-4 text-primary" />
                    <p className="text-2xl font-semibold tracking-tight">{signal.value}</p>
                    <p className="mt-1 text-xs font-medium uppercase tracking-wider text-muted-foreground">{signal.label}</p>
                  </div>
                ))}
              </div>
            </div>

            <div className="relative min-h-[520px]">
              <div className="absolute right-0 top-4 w-full max-w-xl rounded-2xl border border-slate-200/90 bg-white p-3 shadow-[0_24px_70px_rgba(15,23,42,0.08)]">
                <div className="flex items-center justify-between border-b border-slate-200 px-2 pb-3">
                  <div>
                    <p className="text-xs font-medium uppercase tracking-wider text-slate-500">Live operations</p>
                    <p className="text-sm font-semibold text-slate-900">Production control room</p>
                  </div>
                  <StatusBadge status="Down" />
                </div>

                <div className="grid gap-3 pt-3 sm:grid-cols-[1fr_0.72fr]">
                  <div className="space-y-3">
                    {previewMonitors.map((monitor) => (
                      <div key={monitor.name} className="rounded-xl border border-slate-200 bg-slate-50/90 p-3">
                        <div className="flex items-center justify-between gap-3">
                          <p className="truncate text-sm font-medium text-slate-900">{monitor.name}</p>
                          <StatusBadge status={monitor.status} />
                        </div>
                        <div className="mt-3 flex items-center justify-between text-xs text-slate-500">
                          <span>{monitor.region}</span>
                          <span>{monitor.latency}</span>
                        </div>
                      </div>
                    ))}
                  </div>

                  <div className="rounded-xl border border-rose-200 bg-rose-50 p-4">
                    <div className="flex items-center gap-2 text-rose-700">
                      <Clock3 className="size-4" />
                      <span className="text-xs font-medium uppercase tracking-wider">Incident active</span>
                    </div>
                    <p className="mt-4 text-2xl font-semibold text-slate-900">08m 34s</p>
                    <p className="mt-1 text-sm text-slate-600">checkout.service has been unreachable since the last probe.</p>
                    <div className="mt-5 space-y-2 text-xs text-slate-700">
                      <p className="flex items-center gap-2">
                        <CheckCircle2 className="size-3.5 text-emerald-600" />
                        Incident recorded
                      </p>
                      <p className="flex items-center gap-2">
                        <CheckCircle2 className="size-3.5 text-emerald-600" />
                        Webhook dispatched
                      </p>
                      <p className="flex items-center gap-2">
                        <CheckCircle2 className="size-3.5 text-emerald-600" />
                        Status page updated
                      </p>
                    </div>
                  </div>
                </div>
              </div>

              <div className="absolute bottom-3 left-0 w-[92%] max-w-md rounded-2xl border border-slate-200 bg-white p-4 shadow-[0_18px_45px_rgba(15,23,42,0.08)]">
                <div className="flex items-center gap-2">
                  <Bot className="size-4 text-primary" />
                  <p className="text-sm font-semibold">Ask your stack</p>
                </div>
                <div className="mt-3 space-y-2">
                  {assistantPrompts.map((prompt) => (
                    <p key={prompt} className="rounded-md bg-slate-50 px-3 py-2 text-sm text-slate-600">
                      {prompt}
                    </p>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </section>

        <section className="border-b bg-card">
          <div className="mx-auto grid w-full max-w-6xl gap-10 px-4 py-16 lg:grid-cols-[0.72fr_1fr]">
            <div>
              <p className="text-xs font-semibold uppercase tracking-[0.18em] text-primary">Why organizations use it</p>
              <h2 className="mt-3 text-3xl font-semibold tracking-tight text-balance">
                A shared reliability layer for engineering, support, and customer communication.
              </h2>
            </div>
            <div className="grid gap-3">
              {orgBenefits.map((benefit) => (
                <div key={benefit} className="flex gap-3 rounded-lg border bg-background/70 p-4">
                  <CheckCircle2 className="mt-0.5 size-5 shrink-0 text-emerald-600" />
                  <p className="text-sm leading-6 text-muted-foreground">{benefit}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        <section className="bg-background">
          <div className="mx-auto w-full max-w-6xl px-4 py-16">
            <div className="max-w-2xl">
              <p className="text-xs font-semibold uppercase tracking-[0.18em] text-primary">Operating loop</p>
              <h2 className="mt-3 text-3xl font-semibold tracking-tight">
                From signal to response without losing the thread.
              </h2>
            </div>

            <div className="mt-10 grid gap-4 md:grid-cols-2 lg:grid-cols-4">
              {operatingLoop.map((item) => (
                <div key={item.title} className="rounded-lg border bg-card p-5 shadow-sm">
                  <div className="flex size-10 items-center justify-center rounded-md bg-primary/10 text-primary">
                    <item.icon className="size-5" />
                  </div>
                  <h3 className="mt-4 text-base font-semibold">{item.title}</h3>
                  <p className="mt-2 text-sm leading-6 text-muted-foreground">{item.description}</p>
                </div>
              ))}
            </div>
          </div>
        </section>

        <section className="border-y border-slate-200 bg-gradient-to-b from-slate-50 to-white">
          <div className="mx-auto grid w-full max-w-6xl gap-8 px-4 py-16 lg:grid-cols-[1fr_0.9fr]">
            <div>
              <p className="text-xs font-semibold uppercase tracking-[0.18em] text-primary">Automation-ready</p>
              <h2 className="mt-3 text-3xl font-semibold tracking-tight text-balance">
                Connect alerts to the workflows your production teams already use.
              </h2>
              <p className="mt-4 max-w-2xl text-sm leading-7 text-muted-foreground">
                LiveStack can notify a saved email address and call a monitor-specific webhook when
                a service changes state, making it easier to feed deploy systems, chat tools, runbooks,
                or incident pipelines.
              </p>
            </div>
            <div className="grid gap-3 sm:grid-cols-2">
              <div className="rounded-xl border border-slate-200 bg-white p-5 shadow-sm">
                <BellRing className="size-5 text-primary" />
                <h3 className="mt-4 font-semibold">Email status alerts</h3>
                <p className="mt-2 text-sm leading-6 text-muted-foreground">Notify account owners on every monitor status change.</p>
              </div>
              <div className="rounded-xl border border-slate-200 bg-white p-5 shadow-sm">
                <Webhook className="size-5 text-primary" />
                <h3 className="mt-4 font-semibold">Webhook triggers</h3>
                <p className="mt-2 text-sm leading-6 text-muted-foreground">Send structured events to the systems that coordinate response.</p>
              </div>
            </div>
          </div>
        </section>

        <section className="bg-card">
          <div className="mx-auto flex w-full max-w-6xl flex-col items-start justify-between gap-6 px-4 py-14 md:flex-row md:items-center">
            <div>
              <p className="text-xs font-semibold uppercase tracking-[0.18em] text-primary">Start with the services that matter</p>
              <h2 className="mt-3 max-w-2xl text-3xl font-semibold tracking-tight">
                Put your critical APIs, websites, and status communication on one reliable surface.
              </h2>
            </div>
            <Button size="lg" render={<Link to="/signup" />}>
              Create your workspace
              <ArrowRight className="size-4" />
            </Button>
          </div>
        </section>
      </main>

      <footer className="border-t bg-background py-6">
        <div className="mx-auto flex w-full max-w-6xl items-center justify-between px-4 text-sm text-muted-foreground">
          <span>© {new Date().getFullYear()} LiveStack</span>
          <span>Production visibility for teams that ship.</span>
        </div>
      </footer>
    </div>
  )
}
