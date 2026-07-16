import { createContext, useContext, useState, type ReactNode } from "react"

import { clearToken, getToken, setToken as persistToken } from "@/lib/token"

interface AuthContextValue {
  token: string | null
  login: (token: string) => void
  logout: () => void
}

const AuthContext = createContext<AuthContextValue | null>(null)

export function AuthProvider({ children }: { children: ReactNode }) {
  const [token, setToken] = useState<string | null>(() => getToken())

  const login = (newToken: string) => {
    persistToken(newToken)
    setToken(newToken)
  }

  const logout = () => {
    clearToken()
    setToken(null)
  }

  return (
    <AuthContext.Provider value={{ token, login, logout }}>
      {children}
    </AuthContext.Provider>
  )
}

export function useAuth() {
  const ctx = useContext(AuthContext)
  if (!ctx) {
    throw new Error("useAuth must be used within an AuthProvider")
  }
  return ctx
}
