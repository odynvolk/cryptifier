import express from "express";
import http from "http";
import https from "https";
import routes from "./routes";

const setupApp = () => {
  const app = express();

  // Don't limit the number of outgoing HTTP requests (defaults to 4 simultaneous requests)
  http.globalAgent.maxSockets = Infinity;
  https.globalAgent.maxSockets = Infinity;

  // Make sure dates are displayed in the correct timezone
  process.env.TZ = "Europe/Stockholm";

  app.use(routes);

  return app;
};

export default setupApp;
