import nock from "nock";
nock.enableNetConnect(/(localhost|127\.0\.0\.1):\d+/);

process.env.NODE_ENV = "test";

import * as chai from "chai";

chai.config.truncateThreshold = 0;
chai.config.includeStack = true;

export {};
