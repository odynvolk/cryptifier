import axios from "axios";

import logger from "../logger";
import { longCache } from "../cache";

export const getBitnodes = async (): Promise<string> => {
  const value = longCache.get("bitnodes");
  if (value) return value as string;

  try {
    const { data } = await axios.get("https://bitnodes.io/api/v1/snapshots/?limit=1", {
      headers: {
        "content-type": "application/json",
      },
      timeout: 10000,
    });

    const result = data.results[0].total_nodes;
    longCache.set("bitnodes", result);

    logger.debug(`Got reachable nodes from bitnodes.io ${result}`);

    return result;
  } catch (err) {
    logger.error(`Failed to get reachable nodes from bitnodes.io ${err}`);
  }

  return "N/A";
};
