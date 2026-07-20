import { useEffect, useRef, useState } from "react"
import { Check, Download, Loader2, Send } from "lucide-react"
import Markdown from "react-markdown"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { api } from "@/lib/api"
import { exportAssistantTranscriptPdf } from "@/lib/transcript-pdf"
import { cn } from "@/lib/utils"

export interface ChatMessage {
  role: "user" | "assistant"
  content: string
}

/** Mirrors `ai::PendingAction` on the backend (`livestack-backend/ai/src/lib.rs`). */
interface PendingAction {
  name: string
  arguments: unknown
  description: string
}

/** Mirrors `ai::AgentEvent` on the backend (`livestack-backend/ai/src/lib.rs`). */
type AgentEvent =
  | { type: "thinking" }
  | { type: "tool_started"; name: string; arguments: unknown }
  | { type: "tool_finished"; name: string; details: unknown }
  | { type: "confirmation_required"; actions: PendingAction[] }
  | { type: "reply"; content: string }
  | { type: "error"; message: string }

const TOOL_STATUS_LABELS: Record<string, string> = {
  list_websites: "Checking your websites…",
  get_website_metrics: "Analyzing performance metrics…",
  get_incidents: "Looking up incidents…",
  get_status_pages: "Checking status pages…",
  create_website: "Adding the new monitor…",
  update_website: "Updating the monitor…",
  delete_website: "Deleting the monitor…",
}

function toolStatusLabel(name: string): string {
  return TOOL_STATUS_LABELS[name] ?? `Calling ${name.replace(/_/g, " ")}…`
}

const SUGGESTIONS = [
  "Which of my sites are up right now?",
  "Why was my site slow yesterday?",
  "Any incidents this week?",
]

export default function AssistantPage() {
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [input, setInput] = useState("")
  const [sending, setSending] = useState(false)
  const [exporting, setExporting] = useState(false)
  // The agent loop's "thinking" / "calling tool X" trail for the turn in
  // flight, oldest first — appended to, never overwritten, so earlier steps
  // stay visible once later ones arrive.
  const [steps, setSteps] = useState<string[]>([])
  // Mutating action(s) the assistant wants to run, awaiting an explicit
  // confirm/cancel click before anything actually happens.
  const [pending, setPending] = useState<PendingAction[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const scrollRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight })
  }, [messages, sending, steps, pending])

  /**
   * Streams one turn and dispatches its events. Shared by a normal typed
   * message and a confirmed action replay — both just resend the transcript
   * (optionally with `confirmedActions`) and react to the same event types.
   */
  async function runTurn(history: ChatMessage[], confirmedActions?: PendingAction[]) {
    setSending(true)
    setError(null)
    setSteps([])
    setPending(null)

    try {
      for await (const event of api.stream<AgentEvent>("/ai/chat", {
        messages: history,
        confirmed_actions: confirmedActions ?? [],
      })) {
        switch (event.type) {
          case "thinking":
            setSteps((prev) => [...prev, "Thinking…"])
            break
          case "tool_started":
            setSteps((prev) => [...prev, toolStatusLabel(event.name)])
            break
          case "tool_finished":
            // No line of its own: the started step above turns from a
            // spinner into a checkmark once it's no longer the last step.
            break
          case "confirmation_required":
            setPending(event.actions)
            break
          case "reply":
            setMessages([...history, { role: "assistant", content: event.content }])
            break
          case "error":
            setError(event.message)
            break
        }
      }
    } catch {
      setError("The assistant is unavailable right now. Please try again.")
    } finally {
      setSending(false)
      setSteps([])
    }
  }

  async function send(content: string) {
    const trimmed = content.trim()
    if (!trimmed || sending) return

    const history = [...messages, { role: "user" as const, content: trimmed }]
    setMessages(history)
    setInput("")
    await runTurn(history)
  }

  async function confirmPending() {
    if (!pending || sending) return
    await runTurn(messages, pending)
  }

  function cancelPending() {
    if (!pending) return
    setMessages((prev) => [...prev, { role: "assistant", content: "Okay, I won't do that." }])
    setPending(null)
  }

  function handleExportPdf() {
    if (messages.length === 0 || exporting) return
    setExporting(true)
    try {
      exportAssistantTranscriptPdf(messages)
    } finally {
      setExporting(false)
    }
  }

  const pendingIsDestructive = pending?.some((action) => action.name === "delete_website") ?? false

  return (
    <div className="flex h-[calc(100vh-8rem)] w-full flex-col">
      <div className="mb-3 flex shrink-0 items-center justify-between gap-3">
        
        <Button variant="outline" size="sm" onClick={handleExportPdf} disabled={messages.length === 0 || exporting}>
          <Download className="size-4" />
          {exporting ? "Exporting..." : "Export PDF"}
        </Button>
      </div>
      <div ref={scrollRef} className="flex-1 space-y-3 overflow-y-auto">
        {messages.length === 0 && !sending && (
          <div className="flex h-full flex-col items-center justify-center gap-4 text-center">
            <p className="text-sm text-muted-foreground">
              Ask anything about your monitors, incidents, or status pages.
            </p>
            <div className="flex flex-wrap justify-center gap-2">
              {SUGGESTIONS.map((suggestion) => (
                <Button
                  key={suggestion}
                  variant="outline"
                  size="sm"
                  onClick={() => void send(suggestion)}
                >
                  {suggestion}
                </Button>
              ))}
            </div>
          </div>
        )}

        {messages.map((message, i) => (
          <div
            key={i}
            className={cn(
              "max-w-[85%] rounded-lg px-3 py-2 text-sm",
              message.role === "user"
                ? "ml-auto w-fit whitespace-pre-wrap bg-primary text-primary-foreground"
                : "bg-muted",
            )}
          >
            {message.role === "assistant" ? (
              <Markdown
                components={{
                  p: (props) => <p className="mb-2 last:mb-0" {...props} />,
                  ol: (props) => (
                    <ol className="mb-2 list-decimal space-y-1 pl-4 last:mb-0" {...props} />
                  ),
                  ul: (props) => (
                    <ul className="mb-2 list-disc space-y-1 pl-4 last:mb-0" {...props} />
                  ),
                  li: (props) => <li className="[&>ul]:mt-1" {...props} />,
                  strong: (props) => <strong className="font-semibold" {...props} />,
                  a: (props) => (
                    <a
                      className="underline underline-offset-2"
                      target="_blank"
                      rel="noreferrer"
                      {...props}
                    />
                  ),
                  code: (props) => (
                    <code className="rounded bg-background/60 px-1 py-0.5 text-xs" {...props} />
                  ),
                }}
              >
                {message.content}
              </Markdown>
            ) : (
              message.content
            )}
          </div>
        ))}

        {pending && (
          <div className="w-fit max-w-[85%] space-y-2 rounded-lg border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-sm">
            <ul className="space-y-1">
              {pending.map((action, i) => (
                <li key={i}>{action.description}</li>
              ))}
            </ul>
            <div className="flex gap-2">
              <Button
                size="sm"
                variant={pendingIsDestructive ? "destructive" : "default"}
                onClick={() => void confirmPending()}
                disabled={sending}
              >
                Confirm
              </Button>
              <Button size="sm" variant="outline" onClick={cancelPending} disabled={sending}>
                Cancel
              </Button>
            </div>
          </div>
        )}

        {sending && (
          <div className="w-fit space-y-1 rounded-lg bg-muted px-3 py-2 text-sm text-muted-foreground">
            {(steps.length > 0 ? steps : ["Thinking…"]).map((label, i, all) => (
              <div key={i} className="flex items-center gap-2">
                {i === all.length - 1 ? (
                  <Loader2 className="size-4 shrink-0 animate-spin" />
                ) : (
                  <Check className="size-4 shrink-0" />
                )}
                {label}
              </div>
            ))}
          </div>
        )}
        {error && <p className="text-sm text-destructive">{error}</p>}
      </div>

      <form
        className="mt-3 flex shrink-0 items-center gap-2"
        onSubmit={(e) => {
          e.preventDefault()
          void send(input)
        }}
      >
        <Input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Ask about your monitors…"
          disabled={sending}
        />
        <Button type="submit" size="icon" disabled={sending || !input.trim()} aria-label="Send">
          <Send className="size-4" />
        </Button>
      </form>
    </div>
  )
}
