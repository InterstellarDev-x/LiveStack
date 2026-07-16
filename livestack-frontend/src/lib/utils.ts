import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

// Monitoring runs out of the India region, so every timestamp in the UI is
// shown in IST regardless of the viewer's local timezone.
const IST_LOCALE = "en-IN"
const IST_TIME_ZONE = "Asia/Kolkata"

// The API serializes timestamps from a naive (timezone-less) UTC column, e.g.
// "2026-07-14T05:45:57.526" — no "Z", no offset. `new Date(...)` treats a
// string with no timezone designator as *local* time, not UTC, so on a
// machine whose local zone happens to already be IST this would silently
// show the raw UTC clock instead of shifting it. Every timestamp from the API
// must go through this before formatting.
export function toUtcDate(iso: string) {
  const hasTimezone = /[zZ]|[+-]\d{2}:?\d{2}$/.test(iso)
  return new Date(hasTimezone ? iso : `${iso}Z`)
}

export function formatDateTime(iso: string) {
  return toUtcDate(iso).toLocaleString(IST_LOCALE, { timeZone: IST_TIME_ZONE })
}

export function formatTime(iso: string, options?: Intl.DateTimeFormatOptions) {
  return toUtcDate(iso).toLocaleTimeString(IST_LOCALE, { timeZone: IST_TIME_ZONE, ...options })
}

export function formatDate(iso: string, options?: Intl.DateTimeFormatOptions) {
  return toUtcDate(iso).toLocaleDateString(IST_LOCALE, { timeZone: IST_TIME_ZONE, ...options })
}

// "45s", "10m 32s", "2h 05m" — matches the backend's alert-email wording.
export function formatDuration(totalSeconds: number) {
  const seconds = Math.max(0, Math.floor(totalSeconds))
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const rest = seconds % 60

  if (hours > 0) return `${hours}h ${String(minutes).padStart(2, "0")}m`
  if (minutes > 0) return `${minutes}m ${String(rest).padStart(2, "0")}s`
  return `${rest}s`
}
