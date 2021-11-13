import { getPrice } from "./coinPaprika";

const feed = async (req: any, res: any) => {
  const price = await getPrice(req.params.id);
  res.json(price);
};

export default feed;
