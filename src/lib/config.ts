import config from "exp-config";

export const CURRENCIES = typeof config.currencies === "object" ? config.currencies : JSON.parse(config.currencies);
export const CARBON_EMISSIONS_FUTURES = typeof config.carbonEmissionsFutures === "object" ? config.carbonEmissionsFutures : JSON.parse(config.carbonEmissionsFutures || "{}");
