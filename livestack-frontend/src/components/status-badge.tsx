import type { WebsiteStatus } from "@/types/api"

const statusStyles: Record<WebsiteStatus, string> = {
  Up: "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400",
  Down: "bg-destructive/10 text-destructive",
  Unknown: "bg-muted text-muted-foreground",
}

export function StatusBadge({ status }: { status: WebsiteStatus }) {
  return (
    <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${statusStyles[status]}`}>
      {status}
    </span>
  )
}
