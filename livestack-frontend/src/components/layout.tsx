import { Outlet, useNavigate } from "react-router"
import { Activity, LogOut } from "lucide-react"
import { AppSidebar } from "@/components/app-sidebar"
import { Button } from "@/components/ui/button"
import { SidebarInset, SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar"
import { useAuth } from "@/lib/auth"

export function Layout() {
  const { logout } = useAuth()
  const navigate = useNavigate()

  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset className="bg-background">
        <header className="sticky top-0 z-10 flex h-14 shrink-0 items-center gap-3 border-b bg-background/85 px-4 backdrop-blur">
          <SidebarTrigger className="-ml-1 md:hidden" />
          <div className="flex size-8 items-center justify-center rounded-md bg-primary text-primary-foreground">
            <Activity className="size-4" />
          </div>
          <div>
            <span className="text-sm font-semibold">LiveStack</span>
            <p className="text-xs text-muted-foreground">Production operations</p>
          </div>
          <Button
            variant="ghost"
            size="sm"
            className="ml-auto gap-2"
            onClick={() => {
              logout()
              navigate("/signin", { replace: true })
            }}
          >
            <LogOut className="size-4" />
            Log out
          </Button>
        </header>
        <div className="flex-1 overflow-hidden bg-background">
          <main className="mx-auto w-full max-w-7xl p-4 sm:p-6 lg:p-8">
            <Outlet />
          </main>
        </div>
      </SidebarInset>
    </SidebarProvider>
  )
}
