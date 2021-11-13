// @ts-ignore
import config from "exp-config";
import { getTicker } from "./coinGecko";
import notifyTelegram from "./notifiers/telegram";
import { PriceDirection } from "./common";

let lastFloorPrice = 0;

const parseFloorPrice = (price: number) => Math.floor((price / 1000)) * 1000;

const runOnce = async () => {
  const data = await getTicker("bitcoin");
  if (!data?.bitcoin?.usd) {
    return false;
  }

  const price = parseInt(data.bitcoin.usd);
  if (!lastFloorPrice) {
    lastFloorPrice = parseFloorPrice(price);
    return true;
  }

  const currentFloorPrice = parseFloorPrice(price);
  if (currentFloorPrice < lastFloorPrice) {
    lastFloorPrice = currentFloorPrice;
    return await notifyTelegram(price, PriceDirection.DOWN);
  } else if (currentFloorPrice > lastFloorPrice) {
    lastFloorPrice = currentFloorPrice;
    return await notifyTelegram(price, PriceDirection.UP);
  }

  return true;
};

const notifier = async () => {
  await runOnce();
  setInterval(async () => {
    await runOnce();
  }, (config.notifierSleep ?? 120) * 1000);
};

export default notifier;
