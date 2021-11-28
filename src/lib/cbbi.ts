import axios from "axios";
import cache from "./cache";

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
  const value = cache.get("cbbi") as number;
  if (value) return value;

  try {
    const { data } = await axios.get("https://colintalkscrypto.com/cbbi/data/latest.json", {
      headers: {
        "content-type": "application/json",
      },
      timeout: 5000,
    });


    const result = calculateAverage(data);
    console.log("Got CBBI from colintalkscrypto.com", result);
    cache.set("cbbi", result);
  } catch (error) {
    console.log(error);
  }

  return -1;
};
