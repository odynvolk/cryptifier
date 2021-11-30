import logger from "./lib/logger";
import notifier from "./lib/notifier";

logger.info("Cryptifer starting...");

(async () => {
  await notifier();
})();
