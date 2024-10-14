import { getComparison } from "../src/getComparison";

describe("titan-ignore", () => {
  describe("getComparison()", () => {
    it("uses headRelative comparison when not running Khulnasoft CI", async () => {
      expect(getComparison()).toMatchInlineSnapshot(`
        Object {
          "ref": "HEAD^",
          "type": "headRelative",
        }
      `);
    });
    it("returns null when running in Khulnasoft CI with no KHULNASOFT_GIT_PREVIOUS_SHA", async () => {
      process.env.KHULNASOFT = "1";
      expect(getComparison()).toBeNull();
    });

    it("used previousDeploy when running in Khulnasoft CI with KHULNASOFT_GIT_PREVIOUS_SHA", async () => {
      process.env.KHULNASOFT = "1";
      process.env.KHULNASOFT_GIT_PREVIOUS_SHA = "mygitsha";
      expect(getComparison()).toMatchInlineSnapshot(`
        Object {
          "ref": "mygitsha",
          "type": "previousDeploy",
        }
      `);
    });
  });
});
