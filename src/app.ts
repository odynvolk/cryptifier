import logger from "./lib/logger";
import notifier from "./lib/notifier";

logger.info("Cryptifier starting...");

(async () => {
  await notifier();
})();
