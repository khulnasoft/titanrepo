import { createServer } from "./server";
import { log } from "logger";

// eslint-disable-next-line titan/no-undeclared-env-vars
const port = process.env.PORT || 3001;
const server = createServer();

server.listen(port, () => {
  log(`api running on ${port}`);
});
