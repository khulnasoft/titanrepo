#!/usr/bin/env node

import { exec } from "child_process";
import { getTitanRoot, getScopeFromPath, getScopeFromArgs } from "titan-utils";
import { getComparison } from "./getComparison";

console.log(
  "\u226B Using Titanrepo to determine if this project is affected by the commit..."
);

// check for TITAN_FORCE and bail early if it's set
if (process.env.TITAN_FORCE === "true") {
  console.log(
    "\u226B `TITAN_FORCE` detected, skipping check and proceeding with build."
  );
  process.exit(1);
}

// find the monorepo root
const root = getTitanRoot();
if (!root) {
  console.error(
    "Error: workspace root not found. titan-ignore inferencing failed, proceeding with build."
  );
  console.error("");
  process.exit(1);
}

// Find the scope of the project
const argsScope = getScopeFromArgs({ args: process.argv.slice(2) });
const pathScope = getScopeFromPath({ cwd: process.cwd() });
const { context, scope } = argsScope.scope ? argsScope : pathScope;
if (!scope) {
  console.error(
    "Error: app scope not found. titan-ignore inferencing failed, proceeding with build."
  );
  if (!pathScope.scope) {
    console.error(
      'Error: the package.json is missing the "name" field.\nSet this field or pass the --scope flag to titan-ignore.'
    );
  }
  console.error("");
  process.exit(1);
}
if (context.path) {
  console.log(`\u226B Inferred \`${scope}\` as scope from "${context.path}"`);
} else {
  console.log(`\u226B Inferred \`${scope}\` as scope from arguments`);
}

// Get the start of the comparison (previous deployment when available, or previous commit by default)
const comparison = getComparison();
if (!comparison) {
  // This is either the first deploy of the project, or the first deploy for the branch, either way - build it.
  console.log(
    `\u226B No previous deployments found for this project${
      process.env.KHULNASOFT === "1"
        ? ` on "${process.env.KHULNASOFT_GIT_COMMIT_REF}.`
        : "."
    }"`
  );
  console.log(`\u226B Proceeding with build...`);
  process.exit(1);
}
if (comparison.type === "previousDeploy") {
  console.log("\u226B Found previous deployment for project");
}

// Build, and execute the command
const command = `npx titan run build --filter=${scope}...[${comparison.ref}] --dry=json`;
console.log(`\u226B Analyzing results of \`${command}\`...`);
exec(
  command,
  {
    cwd: root,
  },
  (error, stdout) => {
    if (error) {
      console.error(`exec error: ${error}`);
      console.error(`\u226B Proceeding with build to be safe...`);
      process.exit(1);
    }

    try {
      const parsed = JSON.parse(stdout);
      if (parsed == null) {
        console.error(
          `\u226B Failed to parse JSON output from \`${command}\`.`
        );
        console.error(`\u226B Proceeding with build to be safe...`);
        process.exit(1);
      }
      const { packages } = parsed;
      if (packages && packages.length > 0) {
        console.log(
          `\u226B The commit affects this project and/or its ${
            packages.length - 1
          } dependencies`
        );
        console.log(`\u226B Proceeding with build...`);
        process.exit(1);
      } else {
        console.log(
          "\u226B This project and its dependencies are not affected"
        );
        console.log("\u226B Ignoring the change");
        process.exit(0);
      }
    } catch (e) {
      console.error(`\u226B Failed to parse JSON output from \`${command}\`.`);
      console.error(e);
      console.error(`\u226B Proceeding with build to be safe...`);
      process.exit(1);
    }
  }
);
