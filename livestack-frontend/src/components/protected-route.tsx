import { Navigate, Outlet } from "react-router"

import { useAuth } from "@/lib/auth"

export function ProtectedRoute() {
  const { token } = useAuth()

  if (!token) {
    return <Navigate to="/signin" replace />
  }

  return <Outlet />
}
