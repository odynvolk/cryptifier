// @ts-ignore
import ck from "chronokinesis";
// @ts-ignore
import config from "exp-config";
import { expect } from "chai";
import fs from "fs";
import nock from "nock";
import moment from "moment";
import "mocha-cakes-2";
// @ts-ignore
import rewire from "rewire";

import alternativeMe from "../data/alternativeMe.json";
import cbbi from "../data/cbbi.json";

const bitbo = fs.readFileSync("./test/data/bitbo.html");
const blockchainCenter = fs.readFileSync("./test/data/blockchainCenter.html");
const investing = fs.readFileSync("./test/data/investing.html");

const notifier = rewire("../../src/lib/notifier");
const runOnce = notifier.__get__("runOnce");

Feature("Fetch and notify", () => {
  beforeEachScenario(() => {
    nock.cleanAll();
    ck.reset();
  });

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

    and("blockchaincenter.net responds with data for bitcoin raindow chart", () => {
      nock("https://www.blockchaincenter.net")
        .get("/bitcoin-rainbow-chart/")
        .reply(200, blockchainCenter);
    });

    and("alternative.me responds with data for fear and greed index", () => {
      nock("https://api.alternative.me")
        .get("/fng/?limit=1&format=json")
        .reply(200, alternativeMe);
    });

    and("bitbo.io has data", () => {
      nock("https://bitbo.io")
        .get("/")
        .reply(200, bitbo);
    });

    and("investong.com responds with data for carbon emissions futures", () => {
      nock("https://www.investing.com")
        .get("/commodities/carbon-emissions-historical-data/")
        .reply(200, investing);
    });

    and("application fetches price data from CoinGecko", async () => {
      await runOnce();
    });

    given("CoinGecko API has an updated price above increments", () => {
      nock("https://api.coingecko.com")
        .get("/api/v3/simple/price?ids=bitcoin&vs_currencies=usd")
        .reply(200, { bitcoin: { usd: 61221 } });

      nock("https://api.coingecko.com")
        .get("/api/v3/simple/price?ids=ethereum&vs_currencies=usd")
        .reply(200, { ethereum: { usd: 4540 } });
    });

    and("investing.com has an updated price above increments", () => {
      nock("https://www.investing.com")
        .get("/commodities/carbon-emissions-historical-data/")
        .reply(200, investing.toString().replace(/78\.75/g, "79.75"));
    });

    and("Telegram API responds with updates", () => {
      nock("https://api.telegram.org")
        .get(`/bot${config.telegramApiKey}/getUpdates`)
        .times(3)
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
          "chat_id": 123, "parse_mode": "html", text: /Bitcoin is <b>up<\/b>: \$61221/gi,
        })
        .reply(200);

      nock("https://api.telegram.org")
        .post(`/bot${config.telegramApiKey}/sendMessage`, {
          "chat_id": 123, "parse_mode": "html", text: "Ethereum is <b>up</b>! $4540",
        })
        .reply(200);

      nock("https://api.telegram.org")
        .post(`/bot${config.telegramApiKey}/sendMessage`, {
          "chat_id": 123, "parse_mode": "html", text: "Carbon emissions futures are <b>up</b>! €79.75",
        })
        .reply(200);
    });

    when("time passes and caches are flushed", async () => {
      ck.travel(moment().add(12, "h"));
      ck.defrost();
    });

    let result: boolean;
    and("next run", async () => {
      result = await runOnce();
    });

    then("Notifications and everything went for bitcoin, ethereum and carbon emissions futures", () => {
      expect(result).to.deep.equal([true, true, true]);
    });
  });
});
