import fs from "fs-extra";
import chalk from "chalk";
import path from "path";
import { Flags } from "../types";
import { skip, ok, error } from "../logger";

export default function createTurboConfig(files: string[], flags: Flags) {
  if (files.length === 1) {
    const dir = files[0];
    const root = path.resolve(process.cwd(), dir);
    console.log(`Migrating "package.json" "titan" key to "titan.json" file...`);
    const titanConfigPath = path.join(root, "titan.json");
    const rootPackageJsonPath = path.join(root, "package.json");
    let modifiedCount = 0;
    let skippedCount = 0;
    let unmodifiedCount = 2;
    if (!fs.existsSync(rootPackageJsonPath)) {
      error(`No package.json found at ${root}. Is the path correct?`);
      process.exit(1);
    }
    const rootPackageJson = fs.readJsonSync(rootPackageJsonPath);

    if (fs.existsSync(titanConfigPath)) {
      skip("titan.json", chalk.dim("(already exists)"));
      skip("package.json", chalk.dim("(skipped)"));
      skippedCount += 2;
    } else if (rootPackageJson.hasOwnProperty("titan")) {
      const { titan: titanConfig, ...remainingPkgJson } = rootPackageJson;
      if (flags.dry) {
        if (flags.print) {
          console.log(JSON.stringify(titanConfig, null, 2));
        }
        skip("titan.json", chalk.dim("(dry run)"));
        if (flags.print) {
          console.log(JSON.stringify(remainingPkgJson, null, 2));
        }
        skip("package.json", chalk.dim("(dry run)"));
        skippedCount += 2;
      } else {
        if (flags.print) {
          console.log(JSON.stringify(titanConfig, null, 2));
        }
        ok("titan.json", chalk.dim("(created)"));
        fs.writeJsonSync(titanConfigPath, titanConfig, { spaces: 2 });
        if (flags.print) {
          console.log(JSON.stringify(remainingPkgJson, null, 2));
        }
        ok("package.json", chalk.dim("(remove titan key)"));
        fs.writeJsonSync(rootPackageJsonPath, remainingPkgJson, { spaces: 2 });
        modifiedCount += 2;
        unmodifiedCount -= 2;
      }
    } else {
      error('"titan" key does not exist in "package.json"');
      process.exit(1);
    }
    console.log("All done.");
    console.log("Results:");
    console.log(chalk.red(`0 errors`));
    console.log(chalk.yellow(`${skippedCount} skipped`));
    console.log(chalk.yellow(`${unmodifiedCount} unmodified`));
    console.log(chalk.green(`${modifiedCount} modified`));
  }
}
