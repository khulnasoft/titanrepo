export interface Pipeline {
  outputs?: Array<string>;
  dependsOn?: Array<string>;
  inputs?: Array<string>;
}

export interface TitanConfig {
  globalDependencies?: Array<string>;
  pipeline?: Record<string, Pipeline>;
}
