import axios from "axios";
// @ts-ignore
import config from "exp-config";
import logger from "../logger";
import { PriceDirection } from "../common";
import { shortCache } from "../cache";

const getChatIdsFromConfig = () => (config.chatIds?.split(",") || []);

const getChatIds = async () => {
  if (!config.telegramGetUpdates) return getChatIdsFromConfig();

  const value = shortCache.get("getUpdates");
  if (value) return value as string[];

  const { data } = await axios.get(`https://api.telegram.org/bot${config.telegramApiKey}/getUpdates`, {
    timeout: 5000,
  });

  const chatIds = getChatIdsFromConfig().concat(data.result.map((update: any) => update.message.chat.id));
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

const toUpperCase = (ticker: string) => `${ticker.slice(0, 1).toUpperCase()}${ticker.slice(1)}`;

const notify = async (ticker: string, price: number, cbbi: number | null, rainbow: string | null, priceDirection: PriceDirection) => {
  try {
    const chatIds = await getChatIds();
    if (!chatIds) {
      return false;
    }

    const upperCaseTicker = toUpperCase(ticker);
    const priceDirectionText = priceDirection === PriceDirection.UP ? "up" : "down";
    const cbbiText = ticker === "bitcoin" ? ` (CBBI ${cbbi}%)` : "";
    const rainbowText = ticker === "bitcoin" ? ` (Rainbow ${rainbow})` : "";
    const text = `${upperCaseTicker} is <b>${priceDirectionText}</b>! $${price}${cbbiText}${rainbowText}`;
    const textsToSend = chatIds.map((chatId: string) => sendText(chatId, text));

    await Promise.all(textsToSend);

    logger.info(`Notified ${chatIds.length} users about ${upperCaseTicker} price $${price}${cbbiText}${rainbowText}`);
  } catch (err) {
    logger.error(`Failed to notify users of price change! ${err}`);
    return false;
  }

  return true;
};

export default notify;
