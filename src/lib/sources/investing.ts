import axios from "axios";

import logger from "../logger";
import { mediumCache } from "../cache";

const extractPrice = (html: string): string => {
  const m = html.match(/instrument-price-last">[\d.]*<\/span>/) ?? [];
  if (m?.length > 0) {
    const m2 = m[0].match(/\d+.\d+/) ?? [];
    if (m2?.length > 0) return m2[0];
  }

  return "N/A";
};

export const getCarbonEmissionsFuturesPrice = async (): Promise<string> => {
  const value = mediumCache.get("carbonEmissionsFutures");
  if (value) return value as string;

  try {
    const { data } = await axios.get("https://www.investing.com/commodities/carbon-emissions-historical-data/", {
      timeout: 5000,
    });

    const result = extractPrice(data);
    mediumCache.set("carbonEmissionsFutures", result);

    logger.debug(`Got carbon emissions futures price from investing.com €${result}`);

    return result;
  } catch (err) {
    logger.error(`Failed to get carbon emissions futures price from investing.com ${err}`);
  }

  return "N/A";
};
