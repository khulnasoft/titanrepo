import fs from "fs";
import { getTurboRoot } from "titan-utils";
import type { Schema } from "titan-types";

function findTurboConfig({ cwd }: { cwd?: string }): Schema | null {
  const titanRoot = getTurboRoot(cwd);
  if (titanRoot) {
    try {
      const raw = fs.readFileSync(`${titanRoot}/titan.json`, "utf8");
      const titanJsonContent: Schema = JSON.parse(raw);
      return titanJsonContent;
    } catch (e) {
      console.error(e);
      return null;
    }
  }

  return null;
}

export default findTurboConfig;
