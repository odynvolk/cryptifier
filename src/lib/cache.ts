import NodeCache from "node-cache";

const cache = new NodeCache({ stdTTL: 600, checkperiod: 1200 });

export default cache;
