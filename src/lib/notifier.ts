import { getCbbi } from "./cbbi";
// @ts-ignore
import config from "exp-config";
import { getRainbow } from "./blockchainCenter";
import { getTicker } from "./coinGecko";
import notifyTelegram from "./notifiers/telegram";
import logger from "./logger";
import { PriceDirection } from "./common";

const currencies = typeof config.currencies === "object" ? config.currencies : JSON.parse(config.currencies);

const lastFloorPrices = currencies.reduce((acc: any, currency: any) => {
  acc[currency.ticker] = 0;

  return acc;
}, {});

const parseFloorPrice = (price: number, step: number) => Math.floor((price / step)) * step;

const getAndNotify = async (ticker: string, step: number, cbbi: number, rainbow: string) => {
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
    return await notifyTelegram(ticker, price, cbbi, rainbow, PriceDirection.DOWN);
  } else if (currentFloorPrice > lastFloorPrices[ticker]) {
    lastFloorPrices[ticker] = currentFloorPrice;
    return await notifyTelegram(ticker, price, cbbi, rainbow, PriceDirection.UP);
  }

  return true;
};

const runOnce = async () => {
  const [cbbi, rainbow] = await Promise.all([getCbbi(), getRainbow()]);
  const funcs = currencies.map((currency: any) => getAndNotify(currency.ticker, currency.step, cbbi as unknown as number, rainbow as unknown as string));

  return await Promise.all(funcs);
};

const notifier = async () => {
  logger.info(`${currencies.length} currencies defined.`);
  await runOnce();
  setInterval(async () => {
    await runOnce();
  }, (config.notifierSleep ?? 120) * 1000);
};

export default notifier;
