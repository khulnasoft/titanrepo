export function getComparison(): {
  ref: string;
  type: "previousDeploy" | "headRelative";
} | null {
  if (process.env.KHULNASOFT === "1") {
    if (process.env.KHULNASOFT_GIT_PREVIOUS_SHA) {
      // use the commit SHA of the last successful deployment for this project / branch
      return {
        ref: process.env.KHULNASOFT_GIT_PREVIOUS_SHA,
        type: "previousDeploy",
      };
    } else {
      return null;
    }
  }
  return { ref: "HEAD^", type: "headRelative" };
}
