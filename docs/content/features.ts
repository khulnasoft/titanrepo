import React, { ComponentProps } from "react";
import {
  ArrowsExpandIcon,
  BeakerIcon,
  ChartBarIcon,
  ChartPieIcon,
  ChipIcon,
  CloudUploadIcon,
  FingerPrintIcon,
  LightningBoltIcon,
  RefreshIcon,
} from "@heroicons/react/outline";
import { IconType } from "../components/Icons";

export type Feature = {
  name: string;
  description: React.ReactNode;
  Icon: IconType;
  page: "all" | "home" | "docs";
};

export type Features = Array<Feature>;

const FEATURES: Features = [
  {
    name: "Incremental builds",
    description: `Building once is painful enough, Titanrepo will remember what you've built and skip the stuff that's already been computed.`,
    Icon: RefreshIcon,
    page: "all",
  },
  {
    name: "Content-aware hashing",
    description: `Titanrepo looks at the contents of your files, not timestamps to figure out what needs to be built.`,
    Icon: FingerPrintIcon,
    page: "home",
  },
  {
    name: "Parallel execution",
    description: `Execute builds using every core at maximum parallelism without wasting idle CPUs.`,
    Icon: LightningBoltIcon,
    page: "all",
  },
  {
    name: "Remote Caching",
    description: `Share a remote build cache with your teammates and CI/CD for even faster builds.`,
    Icon: CloudUploadIcon,
    page: "all",
  },
  {
    name: "Zero runtime overhead",
    description: `Titanrepo won’t interfere with your runtime code or touch your sourcemaps. `,
    Icon: ChipIcon,
    page: "all",
  },
  {
    name: "Pruned subsets",
    description: `Speed up PaaS deploys by generating a subset of your monorepo with only what's needed to build a specific target.`,
    Icon: ChartPieIcon,
    page: "all",
  },
  {
    name: "Task pipelines",
    description: `Define the relationships between your tasks and then let Titanrepo optimize what to build and when.`,
    Icon: ArrowsExpandIcon,
    page: "all",
  },
  {
    name: "Meets you where you’re at",
    description: `Using Lerna? Keep your package publishing workflow and use Titanrepo to titancharge task running.`,
    Icon: BeakerIcon,
    page: "home",
  },
  {
    name: `Profile in your browser`,
    description: `Generate build profiles and import them in Chrome or Edge to understand which tasks are taking the longest.`,
    Icon: ChartBarIcon,
    page: "home",
  },
];

export const DOCS_FEATURES = FEATURES.filter(
  (f) => f.page === "docs" || f.page === "all"
);

export const HOME_FEATURES = FEATURES.filter(
  (f) => f.page === "home" || f.page === "all"
);

export default FEATURES;
