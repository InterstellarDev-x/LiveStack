import { Outlet, useNavigate } from "react-router"

import { AppSidebar } from "@/components/app-sidebar"
import { Button } from "@/components/ui/button"
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar"
import { useAuth } from "@/lib/auth"

export function Layout() {
  const { logout } = useAuth()
  const navigate = useNavigate()

  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <header className="flex h-12 shrink-0 items-center gap-2 border-b px-4">
          <span className="text-base font-semibold">LiveStack</span>
          <Button
            variant="ghost"
            size="sm"
            className="ml-auto"
            onClick={() => {
              logout()
              navigate("/signin", { replace: true })
            }}
          >
            Log out
          </Button>
        </header>
        <div className="flex-1 p-6">
          <Outlet />
        </div>
      </SidebarInset>
    </SidebarProvider>
  )
}
