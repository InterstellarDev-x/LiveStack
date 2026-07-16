export type WebsiteStatus = "Up" | "Down" | "Unknown"

export interface WebsiteTick {
  id: string
  response_time_ms: number
  status: WebsiteStatus
  region_id: string
  website_id: string
  createdAt: string
  dns_time_ms: number
  connection_time_ms: number
  tls_time_ms: number
  waiting_time_ms: number
  data_transfer_time_ms: number
}

export interface Website {
  id: string
  url: string
  user_id: string
  time_added: string
}

export interface WebsiteWithTick extends Website {
  website_tick: WebsiteTick | null
}

export interface WebsitesByUserOutput {
  websites: Website[]
}

export interface WebsiteTicksOutput {
  ticks: WebsiteTick[]
}

export interface CreateWebsiteOutput {
  success: boolean
  id: string
}

export interface DeleteWebsiteOutput {
  success: boolean
}

export interface SignUpOutput {
  success: boolean
  message: string
}

export interface SignInOutput {
  success: boolean
  token: string
}

export interface WebsiteWebhookConfig {
  webhook_url: string | null
  webhook_secret: string | null
  webhook_enabled: boolean
}

export interface SetWebsiteWebhookOutput extends WebsiteWebhookConfig {
  success: boolean
}

export interface CurrentUser {
  id: string
  username: string
  email: string | null
  email_alerts_enabled: boolean
}

export interface UpdateEmailOutput {
  success: boolean
  email: string
}

export interface UpdateEmailAlertsOutput {
  success: boolean
  email_alerts_enabled: boolean
}

export interface StatusPage {
  id: string
  slug: string
  title: string
  created_at: string
}

export interface StatusPagesOutput {
  pages: StatusPage[]
}

export interface StatusPageMonitor {
  website_id: string
  url: string
  display_name: string
  sort_order: number
}

export interface StatusPageDetail {
  id: string
  slug: string
  title: string
  monitors: StatusPageMonitor[]
}

export interface StatusPageActionOutput {
  success: boolean
}

export interface PublicStatusPageMonitor {
  display_name: string
  status: WebsiteStatus
  uptime_24h: number | null
  uptime_7d: number | null
  uptime_30d: number | null
}

export interface PublicStatusPageIncident {
  display_name: string
  started_at: string
  resolved_at: string | null
  cause: string
}

export interface PublicStatusPage {
  title: string
  monitors: PublicStatusPageMonitor[]
  incidents: PublicStatusPageIncident[]
}

export interface Incident {
  id: string
  website_id: string
  started_at: string
  resolved_at: string | null
  cause: string
  duration_seconds: number | null
}

export interface UserIncident extends Incident {
  url: string
}

export interface WebsiteIncidentsOutput {
  incidents: Incident[]
}

export interface UserIncidentsOutput {
  incidents: UserIncident[]
}
