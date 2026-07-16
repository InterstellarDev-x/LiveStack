import { useEffect, useState } from "react"

import { formatDateTime, toUtcDate } from "@/lib/utils"

function formatElapsed(iso: string) {
  const seconds = Math.max(0, Math.floor((Date.now() - toUtcDate(iso).getTime()) / 1000))

  if (seconds < 60) return `${seconds}s ago`

  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes}m ${seconds % 60}s ago`

  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}h ${minutes % 60}m ago`

  const days = Math.floor(hours / 24)
  return `${days}d ago`
}

/** Renders "Xs/Xm/Xh/Xd ago" for `iso`, ticking on its own every second. */
export function ElapsedTime({ since }: { since: string }) {
  const [label, setLabel] = useState(() => formatElapsed(since))

  useEffect(() => {
    setLabel(formatElapsed(since))
    const id = setInterval(() => setLabel(formatElapsed(since)), 1000)
    return () => clearInterval(id)
  }, [since])

  return <span title={formatDateTime(since)}>{label}</span>
}
