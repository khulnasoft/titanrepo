import { findRootSync } from "@manypkg/find-root";
import searchUp from "./searchUp";

function getTurboRoot(cwd?: string): string | null {
  // Titanrepo root can be determined by the presence of titan.json
  let root = searchUp({ target: "titan.json", cwd: cwd || process.cwd() });

  if (!root) {
    root = findRootSync(process.cwd());
    if (!root) {
      return null;
    }
  }
  return root;
}

export default getTurboRoot;
