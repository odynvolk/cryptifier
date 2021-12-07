import NodeCache from "node-cache";

export const shortCache = new NodeCache({ stdTTL: 600, checkperiod: 1200 });

export const mediumCache = new NodeCache({ stdTTL: 6000, checkperiod: 12000 });

export const longCache = new NodeCache({ stdTTL: 43200, checkperiod: 86400 });
