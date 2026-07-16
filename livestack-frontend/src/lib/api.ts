import { getToken } from "@/lib/token"

const API_BASE = "/api"

export class ApiError extends Error {
  status: number

  constructor(status: number, message: string) {
    super(message)
    this.status = status
  }
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
    throw new ApiError(res.status, `Request to ${path} failed with ${res.status}`)
  }

  if (res.status === 204) {
    return undefined as T
  }

  return (await res.json()) as T
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
    throw new ApiError(res.status, `Request to ${path} failed with ${res.status}`)
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
