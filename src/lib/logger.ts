// @ts-ignore
import config from "exp-config";
import fs from "fs";
import path from "path";
import pino from "pino";
import split from "split2";

const LEVELS = {
  60: "FATAL",
  50: "ERROR",
  40: "WARN",
  30: "INFO",
  20: "DEBUG",
  10: "TRACE",
};

const createStream = (to: any) => {
  const stream = split(mapLine);
  const pipe = stream.pipe;

  stream.pipe = function pipeTo(dest, opts) {
    return pipe.call(stream, dest, opts);
  };

  stream.pipe(to);
  return stream;

  function mapLine(line: string) {
    let time,
      msg,
      err,
      level = null;
    try {
      ({ time, msg, level } = JSON.parse(line)); // eslint-disable-line
    } catch (e) {
      err = e; // eslint-disable-line
    }

    const date = new Date(time);

    if (err) {
      return `${date.toJSON()} LOGERROR ${err.message}\n`;
    }

    // @ts-ignore
    return `${date.toJSON()} ${LEVELS[level]} ${msg}\n`;
  }
};

const getLoggerStream = () => {
  switch (config.log) {
    case "file":
      return createStream(fs.createWriteStream(path.join(__dirname, "..", "logs", `${config.envName}.log`)));
    case "stdout":
      return createStream(process.stdout);
    default:
      throw new Error(`Invalid logger: ${config.log}`);
  }
};

const logger = pino(
  {
    name: "doorkeeper",
    level: config.logLevel,
  },
  getLoggerStream()
);

export default logger;
