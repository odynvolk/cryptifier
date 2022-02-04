// @ts-ignore
import config from "exp-config";

import { getCarbonEmissionsFuturesPrice } from "./sources/investing";
import { getCbbi } from "./sources/cbbi";
import { getFearGreedIndex} from "./sources/alternativeMe";
import { getGrayscalePremium} from "./sources/bitbo";
import { getRainbow } from "./sources/blockchainCenter";
import { getTicker } from "./sources/coinGecko";
import logger from "./logger";
import notifyTelegram from "./notifiers/telegram";
import { PriceChange } from "./common";

const currencies = typeof config.currencies === "object" ? config.currencies : JSON.parse(config.currencies);

const lastFloorPrices = currencies.reduce((acc: any, currency: any) => {
  acc[currency.ticker] = 0;

  return acc;
}, { CFI2Z1: 0 });

const parseFloorPrice = (price: number, increment: number) => Math.floor((price / increment)) * increment;

const getPriceChange = (ticker: string, price: number, increment: number) => {
  if (!lastFloorPrices[ticker]) {
    lastFloorPrices[ticker] = parseFloorPrice(price, increment);
    return PriceChange.NO_CHANGE;
  }

  const currentFloorPrice = parseFloorPrice(price, increment);
  if (currentFloorPrice < lastFloorPrices[ticker]) {
    lastFloorPrices[ticker] = currentFloorPrice;
    return PriceChange.DOWN;
  } else if (currentFloorPrice > lastFloorPrices[ticker]) {
    lastFloorPrices[ticker] = currentFloorPrice;
    return PriceChange.UP;
  }

  return PriceChange.NO_CHANGE;
};

const toUpperCase = (ticker: string) => `${ticker.slice(0, 1).toUpperCase()}${ticker.slice(1)}`;

const getAndNotify = async (ticker: string, increment: number) => {
  const data = await getTicker(ticker);
  if (!data?.[ticker]?.usd) {
    return false;
  }

  const price = parseInt(data?.[ticker]?.usd);
  const priceChange = getPriceChange(ticker, price, increment);
  if (priceChange !== PriceChange.NO_CHANGE) {
    if (ticker === "bitcoin") {
      const [cbbi, rainbow, fgi, gp] = await Promise.all([getCbbi(), getRainbow(), getFearGreedIndex(), getGrayscalePremium()]);
      const text = `Bitcoin is <b>${priceChange}</b>: $${price}\nF&GI: ${fgi}\nCBBI: ${cbbi}%\nRainbow Chart: ${rainbow}\nGrayscale Premium: ${gp}%`;
      return await notifyTelegram(ticker, text);
    }

    const upperCaseTicker = toUpperCase(ticker);
    const text = `${upperCaseTicker} is <b>${priceChange}</b>! $${price}`;
    return await notifyTelegram(ticker, text);
  }

  return false;
};

const getAndNotifyCef = async () => {
  const data = await getCarbonEmissionsFuturesPrice();
  if (!data) {
    return false;
  }

  const ticker = "CFI2Z1";
  const price = parseInt(data);
  const priceChange = getPriceChange(ticker, price, config.carbonEmissionsFutures.increment);
  if (priceChange !== PriceChange.NO_CHANGE) {
    const text = `Carbon emissions futures are <b>${priceChange}</b>! €${data}`;
    return await notifyTelegram(ticker, text);
  }

  return false;
};

const runOnce = async () => {
  const funcs = currencies.map((currency: any) => getAndNotify(currency.ticker, currency.increment));
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
