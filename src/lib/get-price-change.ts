import config from "exp-config";

import { PriceChange } from "./common";
import { CURRENCIES, CARBON_EMISSIONS_FUTURES } from "./config";

const lastFloorPrices = CURRENCIES.reduce((acc: any, { ticker }: { ticker: string }) => {
  acc[ticker] = 0;

  return acc;
}, { [CARBON_EMISSIONS_FUTURES.ticker]: 0 });

const parseFloorPrice = (price: number) => Math.floor((price / 1000)) * 1000;

const getPriceChange = (ticker: string, price: number, increment: number) => {
  const currentFloorPrice = parseFloorPrice(price);
  if (!lastFloorPrices[ticker]) {
    lastFloorPrices[ticker] = currentFloorPrice;
    return PriceChange.NO_CHANGE;
  }

  if (Math.abs(lastFloorPrices[ticker] - currentFloorPrice) > increment) {
    if (currentFloorPrice < lastFloorPrices[ticker]) {
      lastFloorPrices[ticker] = currentFloorPrice;
      return PriceChange.DOWN;
    } else if (currentFloorPrice > lastFloorPrices[ticker]) {
      lastFloorPrices[ticker] = currentFloorPrice;
      return PriceChange.UP;
    }
  }

  return PriceChange.NO_CHANGE;
};

export default getPriceChange;
