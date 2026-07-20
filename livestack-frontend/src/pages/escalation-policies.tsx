import { BellRing, Clock3, Workflow } from "lucide-react"

const policySteps = [
  "Primary responder receives the first status-change alert.",
  "Secondary contact can be added for unresolved incidents.",
  "Webhook routes can connect response workflows to external systems.",
]

export default function EscalationPoliciesPage() {
  return (
    <div className="space-y-6">
      <div className="border-b pb-6">
        <div className="mb-3 inline-flex items-center gap-2 text-xs font-medium text-muted-foreground">
          <Workflow className="size-3.5 text-primary" />
          Response routing
        </div>
        <h1 className="text-3xl font-semibold tracking-tight">Escalation Policies</h1>
        <p className="mt-2 max-w-2xl text-sm leading-6 text-muted-foreground">
          Define how production incidents should move from detection to the people and systems
          responsible for recovery.
        </p>
      </div>

      <section className="py-10 text-center">
        <BellRing className="mx-auto size-8 text-primary" />
        <h2 className="mt-4 text-lg font-semibold">Escalation workflow setup is coming next</h2>
        <p className="mx-auto mt-2 max-w-xl text-sm leading-6 text-muted-foreground">
          LiveStack already supports email alerts and monitor-level webhooks. This page is reserved
          for grouping those routes into reusable escalation policies.
        </p>

        <div className="mx-auto mt-6 grid max-w-3xl gap-6 border-y py-6 text-left md:grid-cols-3">
          {policySteps.map((step) => (
            <div key={step}>
              <Clock3 className="mb-3 size-4 text-primary" />
              <p className="text-sm leading-6 text-muted-foreground">{step}</p>
            </div>
          ))}
        </div>
      </section>
    </div>
  )
}
