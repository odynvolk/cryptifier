import axios from "axios";

export const getTicker = async (id: string) => {
  try {
    const { data } = await axios.get(`https://api.coingecko.com/api/v3/simple/price?ids=${id}&vs_currencies=usd`, {
      headers: {
        "content-type": "application/json",
      },
      timeout: 5000,
    });

    console.log("Got ticker from CoinGecko", data);

    return data;
  } catch (error) {
    console.log(error);
  }

  return null;
};
