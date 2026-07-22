import { Activity } from "lucide-react"

import { cn } from "@/lib/utils"

type BrandMarkProps = {
  className?: string
  iconClassName?: string
}

export function BrandMark({ className, iconClassName }: BrandMarkProps) {
  return (
    <span
      aria-hidden
      className={cn(
        "relative flex size-8 items-center justify-center overflow-hidden rounded-xl border border-cyan-400/25 bg-[linear-gradient(135deg,#0f172a_0%,#0b7285_55%,#14b8a6_100%)] text-white shadow-[0_12px_28px_rgba(20,184,166,0.28)]",
        className,
      )}
    >
      <span className="absolute inset-[3px] rounded-[10px] border border-white/10" />
      <span className="absolute right-1 top-1 size-1.5 rounded-full bg-white/75" />
      <Activity className={cn("relative size-4", iconClassName)} />
    </span>
  )
}
