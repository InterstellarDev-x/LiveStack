import { NavLink, useLocation } from "react-router"
import { Activity, Globe, Settings, Siren, Sparkles, Workflow } from "lucide-react"

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
  { title: "AI Assistant", url: "/assistant", icon: Sparkles },
  { title: "Settings", url: "/settings", icon: Settings },
]

export function AppSidebar() {
  const { pathname } = useLocation()

  return (
    <Sidebar collapsible="icon">
      <SidebarHeader className="flex-row items-center justify-between">
        <SidebarContent className="font-medium">Platform</SidebarContent>
        <SidebarTrigger />
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          
          <SidebarGroupContent>
            <SidebarMenu>
              {navItems.map((item) => (
                <SidebarMenuItem key={item.url}>
                  <SidebarMenuButton
                    isActive={pathname.startsWith(item.url)}
                    tooltip={item.title}
                    render={<NavLink to={item.url} />}
                  >
                    <item.icon />
                    <span>{item.title}</span>
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
