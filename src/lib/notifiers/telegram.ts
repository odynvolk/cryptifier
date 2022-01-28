import axios from "axios";
// @ts-ignore
import config from "exp-config";
import logger from "../logger";
import { shortCache } from "../cache";

const getChatIdsFromConfig = () => (config.telegramChatIds?.split(",") || []);

const getChatIds = async () => {
  if (!config.telegramGetUpdates) return getChatIdsFromConfig();

  const value = shortCache.get("getUpdates");
  if (value) return value as string[];

  const { data } = await axios.get(`https://api.telegram.org/bot${config.telegramApiKey}/getUpdates`, {
    timeout: 5000,
  });

  const chatIds = getChatIdsFromConfig().concat(data.result.map((update: any) => update.message.chat.id));
  logger.debug(`Got chatIds from Telegram: ${JSON.stringify(chatIds)}`);
  const result = [...new Set(chatIds) as unknown as string];
  shortCache.set("getUpdates", result);

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

const notify = async (ticker: string, text: string) => {
  try {
    const chatIds = await getChatIds();
    if (!chatIds) {
      return false;
    }

    const textsToSend = chatIds.map((chatId: string) => sendText(chatId, text));
    await Promise.all(textsToSend);
    logger.info(`Notified ${chatIds.length} users about ${ticker}`);
  } catch (err) {
    logger.error(`Failed to notify users of price change! ${err}`);
    return false;
  }

  return true;
};

export default notify;
