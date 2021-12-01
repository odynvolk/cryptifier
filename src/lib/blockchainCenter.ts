import axios from "axios";
import cheerio from "cheerio";
import logger from "./logger";
import { longCache } from "./cache";

const parsePrice = (price: string) => parseInt(price.replace("$", "").replace("Moon", Number.MAX_SAFE_INTEGER.toString()));

const extractRainbow = (html: string): string | null => {
  const $ = cheerio.load(html);
  const intervals = $("[data-placement=\"top\"]");

  for (let i = 0, len = intervals.length; i < len; i++) {
    const title = intervals.eq(i).attr("title");
    const [floor, ceil, current] = title?.match(/\d*\$|Moon/g) ?? [];
    const pFloor = parsePrice(floor);
    const pCurrent = parsePrice(current);
    const pCeil = parsePrice(ceil);
    if (pFloor < pCurrent && pCurrent < pCeil) {
      return (title?.match(/'.*'/) || [])[0].replace(/'/g, "") ?? null;
    }
  }
  return null;
};

export const getRainbow = async (): Promise<string | null> => {
  const value = longCache.get("rainbow");
  if (value) return value as string;

  try {
    const { data } = await axios.get("https://www.blockchaincenter.net/bitcoin-rainbow-chart/", {
      timeout: 5000,
    });

    const result = extractRainbow(data);
    longCache.set("rainbow", result);

    logger.debug(`Got rainbow from blockchaincenter.net ${result}`);

    return result;
  } catch (err) {
    logger.error(`Failed to get rainbow from blockchaincenter.net ${err}`);
  }

  return null;
};
