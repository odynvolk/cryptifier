import axios from "axios";
import logger from "./logger";
import { longCache } from "./cache";

export const getFearGreedIndex = async (): Promise<string> => {
  const value = longCache.get("f&gi");
  if (value) return value as string;

  try {
    const { data } = await axios.get("https://api.alternative.me/fng/?limit=1&format=json", {
      headers: {
        "content-type": "application/json",
      },
      timeout: 10000,
    });

    const result = `"${data?.data[0]?.value_classification}" \| ${data?.data[0]?.value}`;
    longCache.set("f&gi", result);

    logger.debug(`Got F&GI from api.alternative.me ${result}`);

    return result;
  } catch (err) {
    logger.error(`Failed to get F&GI from api.alternative.me ${err}`);
  }

  return "";
};
