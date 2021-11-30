import axios from "axios";
import logger from "./logger";

export const getTicker = async (id: string) => {
  try {
    const { data } = await axios.get(`https://api.coingecko.com/api/v3/simple/price?ids=${id}&vs_currencies=usd`, {
      headers: {
        "content-type": "application/json",
      },
      timeout: 5000,
    });

    logger.debug(`Got ticker ${id} from CoinGecko`, data);

    return data;
  } catch (err) {
    console.error("Failed to get ticker from CoinGecko", err);
  }

  return null;
};
