import "dotenv/config";

import { Bot } from "grammy";

import { channelAiReply, resolveChannelLink } from "./backend.js";

const token = process.env.TELEGRAM_BOT_TOKEN;
if (!token) throw new Error("TELEGRAM_BOT_TOKEN must be set");

const bot = new Bot(token);

bot.on("message:text", async (ctx) => {
  const chatId = String(ctx.chat.id);

  try {
    const resolved = await resolveChannelLink("telegram", chatId);

    if (!resolved.linked) {
      await ctx.reply(
        `This chat isn't linked to a LiveStack account yet.\n\n` +
          `Go to LiveStack → Integrations and enter this code to link it:\n\n` +
          `${resolved.pairing_code}`
      );
      return;
    }

    await ctx.replyWithChatAction("typing");
    const { reply } = await channelAiReply("telegram", chatId, ctx.message.text);
    await ctx.reply(reply || "(no response)");
  } catch (err) {
    console.error("telegram handler error:", err);
    await ctx.reply("Something went wrong reaching the assistant.");
  }
});

bot.catch((err) => {
  console.error("bot error:", err);
});

bot.start();
console.log("channel-gateway: Telegram bot listening (long polling)");
