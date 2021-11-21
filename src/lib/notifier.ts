// @ts-ignore
import config from "exp-config";
import { getTicker } from "./coinGecko";
import notifyTelegram from "./notifiers/telegram";
import { PriceDirection } from "./common";

const lastFloorPrices = config.currencies.reduce((acc: any, currency: any) => {
  acc[currency.ticker] = 0;

  return acc;
}, {});

const parseFloorPrice = (price: number, step: number) => Math.floor((price / step)) * step;

const getAndNotify = async (ticker: string, step: number) => {
  const data = await getTicker(ticker);
  if (!data?.[ticker]?.usd) {
    return false;
  }

  const price = parseInt(data[ticker].usd);
  if (!lastFloorPrices[ticker]) {
    lastFloorPrices[ticker] = parseFloorPrice(price, step);
    return true;
  }

  const currentFloorPrice = parseFloorPrice(price, step);
  if (currentFloorPrice < lastFloorPrices[ticker]) {
    lastFloorPrices[ticker] = currentFloorPrice;
    return await notifyTelegram(ticker, price, PriceDirection.DOWN);
  } else if (currentFloorPrice > lastFloorPrices[ticker]) {
    lastFloorPrices[ticker] = currentFloorPrice;
    return await notifyTelegram(ticker, price, PriceDirection.UP);
  }

  return true;
};

const runOnce = async () => {
  const funcs = config.currencies.map((currency: any) => getAndNotify(currency.ticker, currency.step));

  return await Promise.all(funcs);
};

const notifier = async () => {
  await runOnce();
  setInterval(async () => {
    await runOnce();
  }, (config.notifierSleep ?? 120) * 1000);
};

export default notifier;
