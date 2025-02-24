// Most of this file is ripped from esbuild
// @see https://github.com/evanw/esbuild/blob/master/lib/npm/node-install.ts
// This file is MIT licensed.

const fs = require("fs");
const os = require("os");
const path = require("path");

// This feature was added to give external code a way to modify the binary
// path without modifying the code itself. Do not remove this because
// external code relies on this.
const TITAN_BINARY_PATH = process.env.TITAN_BINARY_PATH;

const knownWindowsPackages = {
  "win32 arm64 LE": "titan-windows-arm64",
  "win32 x64 LE": "titan-windows-64",
};

const knownUnixlikePackages = {
  "darwin arm64 LE": "titan-darwin-arm64",
  "darwin x64 LE": "titan-darwin-64",
  "linux arm64 LE": "titan-linux-arm64",
  "linux x64 LE": "titan-linux-64",
};

function pkgAndSubpathForCurrentPlatform() {
  let pkg;
  let subpath;
  let platformKey = `${process.platform} ${os.arch()} ${os.endianness()}`;

  if (platformKey in knownWindowsPackages) {
    pkg = knownWindowsPackages[platformKey];
    subpath = "bin/titan.exe";
  } else if (platformKey in knownUnixlikePackages) {
    pkg = knownUnixlikePackages[platformKey];
    subpath = "bin/titan";
  } else {
    throw new Error(`Unsupported platform: ${platformKey}`);
  }
  return { pkg, subpath };
}

function downloadedBinPath(pkg, subpath) {
  const titanLibDir = path.dirname(require.resolve("titan"));
  return path.join(titanLibDir, `downloaded-${pkg}-${path.basename(subpath)}`);
}

function generateBinPath() {
  // This feature was added to give external code a way to modify the binary
  // path without modifying the code itself. Do not remove this because
  // external code relies on this (in addition to titan's own test suite).
  if (TITAN_BINARY_PATH) {
    return TITAN_BINARY_PATH;
  }

  const { pkg, subpath } = pkgAndSubpathForCurrentPlatform();
  let binPath;

  try {
    // First check for the binary package from our "optionalDependencies". This
    // package should have been installed alongside this package at install time.
    binPath = require.resolve(`${pkg}/${subpath}`);
  } catch (e) {
    // If that didn't work, then someone probably installed titan with the
    // "--no-optional" flag. Our install script attempts to compensate for this
    // by manually downloading the package instead. Check for that next.
    binPath = downloadedBinPath(pkg, subpath);
    if (!fs.existsSync(binPath)) {
      // If that didn't work too, then we're out of options. This can happen
      // when someone installs titan with both the "--no-optional" and the
      // "--ignore-scripts" flags. The fix for this is to just not do that.
      //
      // In that case we try to have a nice error message if we think we know
      // what's happening. Otherwise we just rethrow the original error message.
      try {
        require.resolve(pkg);
      } catch {
        throw new Error(`The package "${pkg}" could not be found, and is needed by titan.

If you are installing titan with npm, make sure that you don't specify the
"--no-optional" flag. The "optionalDependencies" package.json feature is used
by titan to install the correct binary executable for your current platform.`);
      }
      throw e;
    }
  }

  // The titan binary executable can't be used in Yarn 2 in PnP mode because
  // it's inside a virtual file system and the OS needs it in the real file
  // system. So we need to copy the file out of the virtual file system into
  // the real file system.
  let isYarnPnP = false;
  try {
    require("pnpapi");
    isYarnPnP = true;
  } catch (e) {}
  if (isYarnPnP) {
    const titanLibDir = path.dirname(require.resolve("titan/package.json"));
    const binTargetPath = path.join(
      titanLibDir,
      `pnpapi-${pkg}-${path.basename(subpath)}`
    );
    if (!fs.existsSync(binTargetPath)) {
      fs.copyFileSync(binPath, binTargetPath);
      fs.chmodSync(binTargetPath, 0o755);
    }
    return binTargetPath;
  }
  return binPath;
}

module.exports = {
  knownUnixlikePackages,
  knownWindowsPackages,
  generateBinPath,
  downloadedBinPath,
  pkgAndSubpathForCurrentPlatform,
  TITAN_BINARY_PATH,
};
