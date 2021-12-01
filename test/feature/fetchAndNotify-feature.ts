// @ts-ignore
import config from "exp-config";
import fs from "fs";
import nock from "nock";
import "mocha-cakes-2";
// @ts-ignore
import rewire from "rewire";
import { expect } from "chai";

import cbbi from "../data/cbbi.json";

const blockchainCenter = fs.readFileSync("./test/data/blockchainCenter.html");

const notifier = rewire("../../src/lib/notifier");
const runOnce = notifier.__get__("runOnce");

Feature("Fetch and notify", () => {
  Scenario("Browsing", () => {
    given("CoinGecko API is up and running", () => {
      nock("https://api.coingecko.com")
        .get("/api/v3/simple/price?ids=bitcoin&vs_currencies=usd")
        .reply(200, { bitcoin: { usd: 60221 } });

      nock("https://api.coingecko.com")
        .get("/api/v3/simple/price?ids=ethereum&vs_currencies=usd")
        .reply(200, { ethereum: { usd: 4386 } });
    });

    and("colintalkscrypto.com responds with data for CBBI", () => {
      nock("https://colintalkscrypto.com")
        .get("/cbbi/data/latest.json")
        .reply(200, cbbi);
    });

    and("blockchaincenter.com responds with data for bitcoin raindow chart", () => {
      nock("https://www.blockchaincenter.net")
        .get("/bitcoin-rainbow-chart/")
        .reply(200, blockchainCenter);
    });

    when("application fetches price data from CoinGecko", async () => {
      await runOnce();
    });

    given("CoinGecko API has an updated price above steps", () => {
      nock("https://api.coingecko.com")
        .get("/api/v3/simple/price?ids=bitcoin&vs_currencies=usd")
        .reply(200, { bitcoin: { usd: 61221 } });

      nock("https://api.coingecko.com")
        .get("/api/v3/simple/price?ids=ethereum&vs_currencies=usd")
        .reply(200, { ethereum: { usd: 4540 } });
    });

    and("Telegram API responds with updates", () => {
      nock("https://api.telegram.org")
        .get(`/bot${config.telegramApiKey}/getUpdates`)
        .times(2)
        .reply(200, {
          "ok": true,
          "result": [
            {
              "update_id": 1234,
              "message": {
                "message_id": 1,
                "from": {
                  "id": 9999,
                  "is_bot": false,
                  "first_name": "Satoshi",
                  "last_name": "Nakamoto",
                  "language_code": "en",
                },
                "chat": {
                  "id": 123,
                  "first_name": "Satoshi",
                  "last_name": "Nakamoto",
                  "type": "private",
                },
                "date": 1636920653,
                "text": "/start",
                "entities": [
                  {
                    "offset": 0,
                    "length": 6,
                    "type": "bot_command",
                  },
                ],
              },
            },
          ],
        });
    });

    and("Telegram accepts notifications to send", () => {
      nock("https://api.telegram.org")
        .post(`/bot${config.telegramApiKey}/sendMessage`, {
          "chat_id": 123, "parse_mode": "html", "text": "Bitcoin is <b>up</b>! $61221 (CBBI 57%) (Rainbow HODL!)",
        })
        .reply(200);

      nock("https://api.telegram.org")
        .post(`/bot${config.telegramApiKey}/sendMessage`, {
          "chat_id": 123, "parse_mode": "html", "text": "Ethereum is <b>up</b>! $4540",
        })
        .reply(200);
    });

    let result: boolean;
    when("next run", async () => {
      result = await runOnce();
    });

    then("Notifications and everything has went well", () => {
      expect(result).to.deep.equal(config.currencies.map(() => true));
    });
  });
});
