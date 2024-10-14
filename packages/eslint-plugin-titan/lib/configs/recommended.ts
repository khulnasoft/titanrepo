import { RULES } from "../constants";

const config = {
  plugins: ["titan"],
  rules: {
    [`titan/${RULES.noUndeclaredEnvVars}`]: "error",
  },
};

export default config;
