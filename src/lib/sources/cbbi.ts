import axios from "axios";

import logger from "../logger";
import { longCache } from "../cache";

const METRICS = ["PiCycle",
  "RUPL",
  "RHODL",
  "Puell",
  "2YMA",
  "Trolololo",
  "MVRV",
  "ReserveRisk",
  "Woobull",
  "HalvingToPeak",
  "GoogleTrends",
  "Confidence",
];

const calculateAverage = (data: any) => {
  const result = METRICS.reduce((acc: number, curr: string) => {
    const metric = data[curr];
    const lastValue = metric[Object.keys(metric)[Object.keys(metric).length - 1]];
    acc += lastValue;

    return acc;
  }, 0);

  return Math.round((result / METRICS.length) * 100);
};

export const getCbbi = async (): Promise<number> => {
  const value = longCache.get("cbbi");
  if (value) return value as number;

  try {
    const { data } = await axios.get("https://colintalkscrypto.com/cbbi/data/latest.json", {
      headers: {
        "content-type": "application/json",
      },
      timeout: 10000,
    });


    const result = calculateAverage(data);
    longCache.set("cbbi", result);

    logger.debug(`Got CBBI from colintalkscrypto.com ${result}%`);

    return result;
  } catch (err) {
    logger.error(`Failed to get CBBI from colintalkscrypto.com ${err}`);
  }

  return -1;
};
