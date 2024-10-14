import findTurboConfig from "./findTurboConfig";
import type { Schema } from "titan-types";

function findDependsOnEnvVars({
  dependencies,
}: {
  dependencies?: Array<string>;
}) {
  if (dependencies) {
    return (
      dependencies
        // filter for dep env vars
        .filter((dep) => dep.startsWith("$"))
        // remove leading $
        .map((envVar) => envVar.slice(1, envVar.length))
    );
  }

  return [];
}

function getEnvVarDependencies({
  cwd,
  titanConfig,
}: {
  cwd: string;
  titanConfig?: Schema;
}): Set<string> | null {
  const titanJsonContent = titanConfig || findTurboConfig({ cwd });
  if (!titanJsonContent) {
    return null;
  }
  const {
    globalDependencies,
    globalEnv = [],
    pipeline = {},
  } = titanJsonContent;

  const allEnvVars: Array<string> = [
    ...findDependsOnEnvVars({
      dependencies: globalDependencies,
    }),
    ...globalEnv,
  ];

  Object.values(pipeline).forEach(({ env, dependsOn }) => {
    if (dependsOn) {
      allEnvVars.push(...findDependsOnEnvVars({ dependencies: dependsOn }));
    }

    if (env) {
      allEnvVars.push(...env);
    }
  });

  return new Set(allEnvVars);
}

export default getEnvVarDependencies;
