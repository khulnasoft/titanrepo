import fs from "fs";
import { getTitanRoot } from "titan-utils";
import type { Schema } from "titan-types";

function findTitanConfig({ cwd }: { cwd?: string }): Schema | null {
  const titanRoot = getTitanRoot(cwd);
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

export default findTitanConfig;
