import { getCbbi } from "./cbbi";
// @ts-ignore
import config from "exp-config";
import { getRainbow } from "./blockchainCenter";
import { getTicker } from "./coinGecko";
import notifyTelegram from "./notifiers/telegram";
import logger from "./logger";
import { PriceDirection } from "./common";
import { getCarbonEmissionsFuturesPrice } from "./investing";

const currencies = typeof config.currencies === "object" ? config.currencies : JSON.parse(config.currencies);

const lastFloorPrices = currencies.reduce((acc: any, currency: any) => {
  acc[currency.ticker] = 0;

  return acc;
}, {});

const parseFloorPrice = (price: number, step: number) => Math.floor((price / step)) * step;

const notify = async (priceStr: string, ticker: string, step: number, cbbi: number | null, rainbow: string | null) => {
  const price = parseInt(priceStr);
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

const getAndNotify = async (ticker: string, step: number) => {
  const data = await getTicker(ticker);
  if (!data?.[ticker]?.usd) {
    return false;
  }

  let cbbi = null, rainbow = null;
  if (ticker === "bitcoin") {
    [cbbi, rainbow] = await Promise.all([getCbbi(), getRainbow()]);
  }

  return notify(data[ticker].usd, ticker, step, cbbi, rainbow);
};

const getAndNotifyCef = async () => {
  const data = await getCarbonEmissionsFuturesPrice();
  if (!data) {
    return false;
  }

  const ticker = "CFI2Z1";
  const { step } = config.carbonEmissionsFutures;

  return notify(data, ticker, step, null, null);
};

const runOnce = async () => {
  const funcs = currencies.map((currency: any) => getAndNotify(currency.ticker, currency.step));
  funcs.push(getAndNotifyCef());

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
