import axios from "axios";

import logger from "../logger";
import { mediumCache } from "../cache";

const extractPrice = (json: any): string => {
  return json.props.children[0].props.children[2].props.children.slice(1) ?? "N/A";
};

export const getCarbonEmissionsFuturesPrice = async (): Promise<string> => {
  const value = mediumCache.get("carbonEmissionsFutures");
  if (value) return value as string;

  try {
    const { data } = await axios.get("https://sandbag-carbon-price-viewer.herokuapp.com/_dash-layout", {
      timeout: 5000,
    });

    const result = extractPrice(data);
    mediumCache.set("carbonEmissionsFutures", result);

    logger.debug(`Got carbon emissions futures price from sandbag.be €${result}`);

    return result;
  } catch (err) {
    logger.error(`Failed to get carbon emissions futures price from sandbag.be ${err}`);
  }

  return "N/A";
};
