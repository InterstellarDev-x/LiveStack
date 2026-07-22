import { NavLink, useLocation } from "react-router"
import { Activity, Globe, Radar, Settings, ShieldCheck, Siren, Sparkles, Workflow } from "lucide-react"

import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarTrigger,
} from "@/components/ui/sidebar"

const navItems = [
  { title: "Monitors", url: "/monitors", icon: Activity },
  { title: "Incidents", url: "/incidents", icon: Siren },
  { title: "Escalation Policies", url: "/escalation-policies", icon: Workflow },
  { title: "Status Pages", url: "/status-pages", icon: Globe },
  { title: "Network Tools", url: "/network-tools", icon: Radar },
  { title: "AI Assistant", url: "/assistant", icon: Sparkles },
  { title: "Settings", url: "/settings", icon: Settings },
]

export function AppSidebar() {
  const { pathname } = useLocation()

  return (
    <Sidebar collapsible="icon" className="border-r bg-background">
      <SidebarHeader className="border-b px-3 py-3">
        <SidebarContent className="min-w-0">
          <div className="flex items-center justify-between gap-2 rounded-lg bg-muted px-2 py-2 text-muted-foreground group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:bg-transparent group-data-[collapsible=icon]:p-0">
            <div className="flex min-w-0 items-center gap-2 group-data-[collapsible=icon]:hidden">
              <ShieldCheck className="size-4 shrink-0" />
              <div className="min-w-0">
                <p className="truncate text-sm font-semibold text-foreground">Operations</p>
                <p className="truncate text-xs text-muted-foreground">Monitor and respond</p>
              </div>
            </div>
            <SidebarTrigger />
          </div>
        </SidebarContent>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup className="px-2 py-3">
          <SidebarGroupContent>
            <SidebarMenu className="gap-1">
              {navItems.map((item) => (
                <SidebarMenuItem key={item.url}>
                  <SidebarMenuButton
                    isActive={pathname.startsWith(item.url)}
                    tooltip={item.title}
                    className="h-10 rounded-md data-[active=true]:bg-accent data-[active=true]:text-accent-foreground"
                    render={<NavLink to={item.url} />}
                  >
                    <item.icon className="size-4" />
                    <span className="font-medium">{item.title}</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
    </Sidebar>
  )
}
