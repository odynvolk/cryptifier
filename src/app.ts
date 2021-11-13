import setupApp from "./lib/init/setupApp";

const app = setupApp();

// Only listen if started, not if included
if (require.main === module) {
  const port = Number(process.env.PORT) || 3000;
  const server = app.listen(port, () => {
  });

  const exitRouter = (options: any, exitCode: number | string) => {
    // eslint-disable-next-line no-console
    if (exitCode || exitCode === 0) console.log(`${exitCode === "SIGINT" ? "\n" : ""}Exit: ${exitCode}`);
    if (options.exit) process.exit(1); // eslint-disable-line no-process-exit
  };

  const exitHandler = (exitCode: any) => {
    server.close(exitCode);
  };

  const others = ["SIGINT", "SIGUSR1", "SIGUSR2", "SIGHUP", "uncaughtException", "SIGTERM"];
  others.forEach((eventType) => {
    process.on(eventType, exitRouter.bind(null, { exit: true }));
  });

  process.on("exit", exitHandler);
}

export default app;
