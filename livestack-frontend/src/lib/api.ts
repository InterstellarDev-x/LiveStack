import { clearToken, getToken } from "@/lib/token"

const API_BASE = import.meta.env.VITE_API_BASE ?? "/api"

/**
 * Endpoints where a 401 is the answer to the question being asked ("are these
 * credentials right?") rather than a sign the session has expired. Everywhere
 * else, a 401 means the token is gone or stale and the user needs to sign in
 * again.
 */
const AUTH_PATHS = ["/signin", "/signup"]

export class ApiError extends Error {
  status: number

  constructor(status: number, message: string) {
    super(message)
    this.status = status
  }
}

/**
 * Tokens expire (10 hours), and nothing on the client watches for that. Left
 * alone, every page just renders its generic "Couldn't load..." error forever
 * while the user sits on a signed-in-looking app. Drop the dead token and send
 * them to sign-in instead.
 */
function handleExpiredSession(path: string) {
  if (AUTH_PATHS.includes(path) || !getToken()) return

  clearToken()
  // A full navigation, not a router push: this can fire from anywhere,
  // including outside a component, and it should clear all in-memory state.
  window.location.assign("/signin")
}

/** Parses a response body, tolerating handlers that return no content. */
async function parseBody<T>(res: Response): Promise<T> {
  if (res.status === 204) {
    return undefined as T
  }

  // Some handlers (e.g. DELETE /channels/links/:id) answer 200 with an empty
  // body. Calling res.json() on that throws, which used to surface as a
  // failure for an operation that had actually succeeded.
  const text = await res.text()
  if (text.trim() === "") {
    return undefined as T
  }

  return JSON.parse(text) as T
}

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const token = getToken()

  const res = await fetch(`${API_BASE}${path}`, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...(token ? { token } : {}),
      ...options.headers,
    },
  })

  if (!res.ok) {
    if (res.status === 401) {
      handleExpiredSession(path)
    }
    throw new ApiError(res.status, `Request to ${path} failed with ${res.status}`)
  }

  return parseBody<T>(res)
}

/**
 * POSTs `body` and yields server-sent events as they arrive, instead of
 * waiting for the response to finish. Each SSE frame's `data:` line is
 * parsed as JSON and yielded — the `event:` field is ignored since our
 * payloads already carry their own `type` discriminant.
 */
async function* stream<T>(path: string, body?: unknown): AsyncGenerator<T> {
  const token = getToken()

  const res = await fetch(`${API_BASE}${path}`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...(token ? { token } : {}),
    },
    body: body !== undefined ? JSON.stringify(body) : undefined,
  })

  if (!res.ok || !res.body) {
    const detail = await res.text().catch(() => "")
    throw new ApiError(res.status, detail || `Request to ${path} failed with ${res.status}`)
  }

  const reader = res.body.getReader()
  const decoder = new TextDecoder()
  let buffer = ""

  while (true) {
    const { done, value } = await reader.read()
    if (done) break
    buffer += decoder.decode(value, { stream: true })

    let boundary: number
    while ((boundary = buffer.indexOf("\n\n")) !== -1) {
      const frame = buffer.slice(0, boundary)
      buffer = buffer.slice(boundary + 2)

      const dataLines = frame
        .split("\n")
        .filter((line) => line.startsWith("data:"))
        .map((line) => line.slice(5).trim())
      if (dataLines.length === 0) continue // comment/keep-alive frame

      try {
        yield JSON.parse(dataLines.join("\n")) as T
      } catch {
        // malformed frame; skip rather than break the whole stream
      }
    }
  }
}

export const api = {
  get: <T>(path: string) => request<T>(path),
  post: <T>(path: string, body?: unknown) =>
    request<T>(path, {
      method: "POST",
      body: body !== undefined ? JSON.stringify(body) : undefined,
    }),
  put: <T>(path: string, body?: unknown) =>
    request<T>(path, {
      method: "PUT",
      body: body !== undefined ? JSON.stringify(body) : undefined,
    }),
  patch: <T>(path: string, body?: unknown) =>
    request<T>(path, {
      method: "PATCH",
      body: body !== undefined ? JSON.stringify(body) : undefined,
    }),
  delete: <T>(path: string) => request<T>(path, { method: "DELETE" }),
  stream,
}
