// @ts-ignore
import config from "exp-config";

import { CURRENCIES, CARBON_EMISSIONS_FUTURES } from "./config";
import { getBitnodes } from "./sources/bitnodes";
import { getCarbonEmissionsFuturesPrice } from "./sources/sandbag";
import { getCbbi } from "./sources/cbbi";
import { getFearGreedIndex } from "./sources/alternative-me";
import getPriceChange from "./get-price-change";
import { getTicker } from "./sources/coin-gecko";
import logger from "./logger";
import notifyTelegram from "./notifiers/telegram";
import { PriceChange } from "./common";

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
      const [cbbi, fgi, bitnodes] = await Promise.all([getCbbi(), getFearGreedIndex(), getBitnodes()]);
      const text = `Bitcoin is <b>${priceChange}</b>: $${price}\nReachable nodes: ${bitnodes}\nF&GI: ${fgi}\nCBBI: ${cbbi}%`;
      return await notifyTelegram(ticker, text);
    }

    const upperCaseTicker = toUpperCase(ticker);
    const text = `${upperCaseTicker} is <b>${priceChange}</b>! $${price}`;
    return await notifyTelegram(ticker, text);
  }

  return false;
};

const getAndNotifyCef = async () => {
  if (!CARBON_EMISSIONS_FUTURES.ticker) return false;

  const data = await getCarbonEmissionsFuturesPrice();
  if (!data) {
    return false;
  }

  const price = parseInt(data);
  const priceChange = getPriceChange(CARBON_EMISSIONS_FUTURES.ticker, price, CARBON_EMISSIONS_FUTURES.increment);
  if (priceChange !== PriceChange.NO_CHANGE) {
    const text = `Carbon emissions futures are <b>${priceChange}</b>! €${data}`;
    return await notifyTelegram(CARBON_EMISSIONS_FUTURES.ticker, text);
  }

  return false;
};

const runOnce = async () => {
  const funcs = CURRENCIES.map((currency: any) => getAndNotify(currency.ticker, currency.increment));
  return await Promise.all([...funcs, getAndNotifyCef()]);
};

const notifier = async () => {
  logger.info(`${CURRENCIES.length} currencies defined.`);
  await runOnce();

  setInterval(async () => {
    await runOnce();
  }, (config.notifierSleep ?? 120) * 1000);
};

export default notifier;
