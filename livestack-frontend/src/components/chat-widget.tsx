import { useEffect, useRef, useState } from "react"
import { Loader2, MessageCircle, Send, X } from "lucide-react"
import Markdown from "react-markdown"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { api } from "@/lib/api"
import { cn } from "@/lib/utils"

interface ChatMessage {
  role: "user" | "assistant"
  content: string
}

interface AiChatOutput {
  reply: string
}

const GREETING: ChatMessage = {
  role: "assistant",
  content:
    "Hi! I can answer questions about your monitors — try \"why was my site slow yesterday?\" or \"any incidents this week?\"",
}

export function ChatWidget() {
  const [open, setOpen] = useState(false)
  const [messages, setMessages] = useState<ChatMessage[]>([GREETING])
  const [input, setInput] = useState("")
  const [sending, setSending] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const scrollRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight })
  }, [messages, sending, open])

  async function send() {
    const content = input.trim()
    if (!content || sending) return

    // The greeting is UI-only; the API gets the real turns.
    const history = [...messages.filter((m) => m !== GREETING), { role: "user" as const, content }]
    setMessages([GREETING, ...history])
    setInput("")
    setSending(true)
    setError(null)

    try {
      const res = await api.post<AiChatOutput>("/ai/chat", { messages: history })
      setMessages([GREETING, ...history, { role: "assistant", content: res.reply }])
    } catch {
      setError("The assistant is unavailable right now. Please try again.")
    } finally {
      setSending(false)
    }
  }

  if (!open) {
    return (
      <Button
        size="icon"
        className="fixed bottom-6 right-6 z-50 size-12 rounded-full shadow-lg"
        onClick={() => setOpen(true)}
        aria-label="Open assistant"
      >
        <MessageCircle className="size-5" />
      </Button>
    )
  }

  return (
    <div className="fixed bottom-6 right-6 z-50 flex h-[28rem] w-80 flex-col overflow-hidden rounded-xl border bg-background shadow-xl">
      <header className="flex h-11 shrink-0 items-center justify-between border-b px-3">
        <span className="text-sm font-semibold">LiveStack Assistant</span>
        <Button
          variant="ghost"
          size="icon"
          className="size-7"
          onClick={() => setOpen(false)}
          aria-label="Close assistant"
        >
          <X className="size-4" />
        </Button>
      </header>

      <div ref={scrollRef} className="flex-1 space-y-3 overflow-y-auto p-3">
        {messages.map((message, i) => (
          <div
            key={i}
            className={cn(
              "max-w-[85%] rounded-lg px-3 py-2 text-sm",
              message.role === "user"
                ? "ml-auto whitespace-pre-wrap bg-primary text-primary-foreground"
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
        {sending && (
          <div className="flex items-center gap-2 rounded-lg bg-muted px-3 py-2 text-sm text-muted-foreground">
            <Loader2 className="size-4 animate-spin" />
            Checking your monitors…
          </div>
        )}
        {error && <p className="text-sm text-destructive">{error}</p>}
      </div>

      <form
        className="flex shrink-0 items-center gap-2 border-t p-2"
        onSubmit={(e) => {
          e.preventDefault()
          void send()
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
