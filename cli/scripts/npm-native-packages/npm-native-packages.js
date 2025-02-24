#!/usr/bin/env node

const fs = require("fs");
const path = require("path");

// Map to node os and arch names.
const nodeOsLookup = {
  darwin: "darwin",
  linux: "linux",
  windows: "win32",
};

const nodeArchLookup = {
  amd64: "x64",
  arm64: "arm64",
};

const humanizedArchLookup = {
  amd64: "64",
  arm64: "arm64",
};

const template = require("./template/template.package.json");
const os = process.argv[2];
const arch = process.argv[3];
const version = process.argv[4];

template.name = `titan-${os}-${humanizedArchLookup[arch]}`;
template.description = `The ${os}-${humanizedArchLookup[arch]} binary for titan, a monorepo build system.`;
template.os = [nodeOsLookup[os]];
template.cpu = [nodeArchLookup[arch]];
template.version = version;

const outputPath = path.join(__dirname, "build", template.name);
fs.rmSync(outputPath, { recursive: true, force: true });
fs.mkdirSync(outputPath, { recursive: true });

if (os === "windows") {
  fs.mkdirSync(path.join(outputPath, "bin"));
  fs.copyFileSync(
    path.join(__dirname, "template", "bin", "titan"),
    path.join(outputPath, "bin", "titan")
  );
}
fs.copyFileSync(
  path.join(__dirname, "template", "README.md"),
  path.join(outputPath, "README.md")
);
fs.writeFileSync(
  path.join(outputPath, "package.json"),
  JSON.stringify(template, null, 2)
);
