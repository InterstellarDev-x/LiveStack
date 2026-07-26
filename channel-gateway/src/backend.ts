const API_BASE_URL = process.env.API_BASE_URL ?? "http://localhost:3000";
const INTERNAL_API_SECRET = requiredEnv("INTERNAL_API_SECRET");

function requiredEnv(name: string): string {
  const value = process.env[name];
  if (!value) throw new Error(`${name} must be set`);
  return value;
}

async function internalPost<T>(path: string, body: unknown): Promise<T> {
  const res = await fetch(`${API_BASE_URL}${path}`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "x-internal-secret": INTERNAL_API_SECRET,
    },
    body: JSON.stringify(body),
  });

  if (!res.ok) {
    const detail = await res.text().catch(() => "");
    throw new Error(`${path} failed with ${res.status}: ${detail}`);
  }

  return (await res.json()) as T;
}

export interface ResolveChannelLinkResult {
  linked: boolean;
  pairing_code: string | null;
}

export function resolveChannelLink(
  channel: string,
  channelUserId: string
): Promise<ResolveChannelLinkResult> {
  return internalPost("/internal/channel-link/resolve", {
    channel,
    channel_user_id: channelUserId,
  });
}

export interface ChannelAiReplyResult {
  reply: string;
}

export function channelAiReply(
  channel: string,
  channelUserId: string,
  message: string
): Promise<ChannelAiReplyResult> {
  return internalPost("/internal/ai/reply", {
    channel,
    channel_user_id: channelUserId,
    message,
  });
}
