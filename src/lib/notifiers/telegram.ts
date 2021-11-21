import axios from "axios";
// @ts-ignore
import config from "exp-config";
import { PriceDirection } from "../common";

import NodeCache from "node-cache";

const cache = new NodeCache({ stdTTL: 600, checkperiod: 1200 });

const getChatIdsFromConfig = () => (config.chatIds?.split(",") || []);

const getChatIds = async () => {
  if (!config.telegramGetUpdates) return getChatIdsFromConfig();

  const value = cache.get("getUpdates");
  if (value) return value as string[];

  const { data } = await axios.get(`https://api.telegram.org/bot${config.telegramApiKey}/getUpdates`, {
    timeout: 5000,
  });

  const chatIds = getChatIdsFromConfig().concat(data.result.map((update: any) => update.message.chat.id));
  const result = [...new Set(chatIds) as unknown as string];
  cache.set("getUpdates", result);

  return result;
};

const sendText = async (chatId: string, text: string) => {
  await axios.post(`https://api.telegram.org/bot${config.telegramApiKey}/sendMessage`, {
    chat_id: chatId,
    parse_mode: "html",
    text,
  }, {
    headers: {
      "content-type": "application/json",
    },
    timeout: 5000,
  });
};

const toUpperCase = (ticker: string) => `${ticker.slice(0, 1).toUpperCase()}${ticker.slice(1)}`;

const notify = async (ticker: string, price: number, priceDirection: PriceDirection) => {
  try {
    const chatIds = await getChatIds();
    if (!chatIds) {
      return false;
    }

    const text = `${toUpperCase(ticker)} is <b>${priceDirection === PriceDirection.UP ? "up" : "down"}</b>! $${price}`;
    const textsToSend = chatIds.map((chatId: string) => sendText(chatId, text));

    await Promise.all(textsToSend);

    console.log(`Notified ${chatIds.length} users about price $${price}`);
  } catch (err) {
    console.log("Failed to notify users of price change!", err);
    return false;
  }

  return true;
};

export default notify;
