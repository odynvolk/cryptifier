import axios from "axios";
import NodeCache from "node-cache";

const cache = new NodeCache( { stdTTL: 60 } )

const headers = {
  "content-type": "application/json",
};

export const getPrice = async (id: string) => {
  try {
    let data: any = cache.get( id );
    if (data) {
      console.log("@@@ coinPaprika.ts getPrice 14");
      return data;
    }

    ({ data } = await axios.get(`https://api.coinpaprika.com/v1/tickers/${id}`, {
      headers,
      timeout: 5000,
    }));

    cache.set(id, data);

    console.log("@@@ coinPaprika.ts getPrice 16", JSON.stringify(data, null, 2));
    console.log("@@@ coinPaprika.ts getPrice 17", data.quotes.USD.price);
    return data;
  } catch (error) {
    console.log(error);
  }
}
