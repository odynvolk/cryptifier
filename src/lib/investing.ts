import axios from "axios";
import cheerio from "cheerio";
import logger from "./logger";
import { mediumCache } from "./cache";

const extractPrice = (html: string): string | null => {
  const $ = cheerio.load(html);
  const lastPrice = $("#_last_8848");

  return lastPrice.text();
};

export const getCarbonEmissionsFuturesPrice = async (): Promise<string | null> => {
  const value = mediumCache.get("carbonEmissionsFutures");
  if (value) return value as string;

  try {
    const { data } = await axios.get("https://www.investing.com/commodities/carbon-emissions-historical-data/", {
      timeout: 5000,
    });

    const result = extractPrice(data);
    mediumCache.set("carbonEmissionsFutures", result);

    logger.debug(`Got carbon emissions futures price from investing.com ${result}`);

    return result;
  } catch (err) {
    logger.error(`Failed to get carbon emissions futures price from investing.com ${err}`);
  }

  return null;
};
