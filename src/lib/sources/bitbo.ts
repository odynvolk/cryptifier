import axios from "axios";

import logger from "../logger";
import { mediumCache } from "../cache";

const extractPremium = (html: string): string => {
  const idx = html.indexOf("GBTC Premium");
  const match = html.slice(idx, idx + 90).match(/-\d{2}\.\d{2}/) ?? [];
  if (match?.length > 0) return match[0] ?? "N/A";

  return "N/A";
};

export const getGrayscalePremium = async (): Promise<string> => {
  const value = mediumCache.get("bitbo");
  if (value) return value as string;

  try {
    const { data } = await axios.get("https://bitbo.io/", {
      timeout: 5000,
    });

    const result = extractPremium(data);
    mediumCache.set("bitbo", result);

    logger.debug(`Got data from bitbo.io ${result}%`);

    return result;
  } catch (err) {
    logger.error(`Failed to get data from bitbo.io ${err}`);
  }

  return "N/A";
};
