import copy from "copy-to-clipboard";
import Head from "next/head";
import Link from "next/link";
import toast, { Toaster } from "react-hot-toast";

import { Container } from "../Container";
import { HomeFeatures } from "../Features";

export default function Home() {
  const onClick = () => {
    copy("npx create-titan@latest");
    toast.success("Copied to clipboard");
  };

  return (
    <>
      <Head>
        <title>Titanrepo</title>
        <meta
          name="og:description"
          content="Titanrepo is a high-performance build system for JavaScript and
          TypeScript codebases"
        />
      </Head>
      <div className="w-auto px-4 pt-16 pb-8 mx-auto sm:pt-24 lg:px-8">
        <h1 className="max-w-5xl text-center mx-auto text-6xl font-extrabold tracking-tighter leading-[1.1] sm:text-7xl lg:text-8xl xl:text-8xl">
          The build system that
          <br className="hidden lg:block" />
          <span className="inline-block text-transparent bg-clip-text bg-gradient-to-r from-pink-gradient-start to-blue-500 ">
            makes ship happen.
          </span>{" "}
        </h1>
        <p className="max-w-lg mx-auto mt-6 text-xl font-medium leading-tight text-center text-gray-400 sm:max-w-4xl sm:text-2xl md:text-3xl lg:text-4xl">
          Titanrepo is a high-performance build system for JavaScript and
          TypeScript codebases.
        </p>
        <div className="max-w-xl mx-auto mt-5 sm:flex sm:justify-center md:mt-8">
          <div className="rounded-md ">
            <Link href="/docs/getting-started">
              <a className="flex min-w-[120px] items-center justify-center w-full px-8 py-3 text-base font-medium text-white no-underline bg-black border border-transparent rounded-md dark:bg-white dark:text-black betterhover:dark:hover:bg-gray-300 betterhover:hover:bg-gray-700 md:py-3 md:text-lg md:px-10 md:leading-6">
                Start Building →
              </a>
            </Link>
          </div>
          <div className="relative mt-3 rounded-md sm:mt-0 sm:ml-3">
            <a
              onClick={onClick}
              className="flex min-w-[200px] items-center justify-center w-full px-8 py-3  text-base font-medium text-gray-600 bg-black border border-transparent border-gray-200 rounded-md bg-opacity-5 dark:bg-white dark:text-gray-300 dark:border-gray-700 dark:bg-opacity-5 betterhover:hover:bg-gray-50 betterhover:dark:hover:bg-gray-900 md:py-3 md:text-base md:leading-6 md:px-10"
            >
              GitHub
            </a>
          </div>
        </div>
      </div>

      <div className="py-16">
        <div className="mx-auto ">
          <p className="pb-8 text-sm font-semibold tracking-wide text-center text-gray-400 uppercase dark:text-gray-500">
            Trusted by teams from around the world
          </p>
        </div>
      </div>

      <div className="relative from-gray-50 to-gray-100">
        <div className="px-4 py-16 mx-auto sm:pt-20 sm:pb-24 lg:max-w-7xl lg:pt-24">
          <h2 className="text-4xl font-extrabold tracking-tight lg:text-5xl xl:text-6xl lg:text-center dark:text-white">
            Build like the best
          </h2>
          <p className="mx-auto mt-4 text-lg font-medium text-gray-400 lg:max-w-3xl lg:text-xl lg:text-center">
            Titanrepo reimagines build system techniques used by Facebook and
            Google to remove maintenance burden and overhead.
          </p>
          <HomeFeatures />
        </div>
      </div>
      <div className="">
        <div className="px-4 py-16 mx-auto sm:pt-20 sm:pb-24 lg:pt-24 lg:px-8">
          <h2 className="max-w-4xl mx-auto pb-6 text-5xl font-extrabold  tracking-tight lg:text-6xl xl:text-7xl leading-[1.25!important] md:text-center dark:text-white">
            Scaling your codebase shouldn&apos;t be so difficult
          </h2>
          <div className="max-w-2xl mx-auto lg:mt-2 dark:text-gray-400">
            <p className="mb-6 text-lg leading-normal text-current lg:text-xl">
              The bigger your project grows, the slower it gets. Tasks like
              linting, testing, and building begin to take enormous amounts of
              time.
            </p>
            <p className="mb-6 text-lg leading-normal text-current lg:text-xl">
              If you&apos;re serving multiple applications, you might reach for
              a monorepo. They&apos;re incredible for productivity, especially
              on the frontend, but the tooling can be a nightmare. There&apos;s
              a lot of stuff to do (and things to mess up). Nothing &ldquo;just
              works.&rdquo; It&apos;s become completely normal to waste entire
              days or weeks on plumbing—tweaking configs, writing one-off
              scripts, and stitching stuff together.
            </p>
            <p className="mb-6 text-lg leading-normal text-current lg:text-xl">
              We need something else.
            </p>
            <p className="mb-6 text-lg leading-normal text-current lg:text-xl">
              A fresh take on the whole setup. Designed to glue everything
              together. A toolchain that works for you and not against you. With
              sensible defaults, but even better escape hatches. Built with the
              same techniques used by the big guys, but in a way that
              doesn&apos;t require PhD to learn or a staff to maintain.
            </p>
            <p className="mb-6 text-lg leading-normal text-current lg:text-xl">
              <b className="relative inline-block text-transparent bg-clip-text bg-gradient-to-r from-pink-gradient-start to-blue-500">
                With Titanrepo, we&apos;re doing just that.
              </b>{" "}
              We&apos;re building a build system that can keep up with your
              team. You&apos;ll see your CI get faster, duplicated work get cut,
              and your NPM scripts get simpler. You&apos;ll get a world-class
              development environment, without the maintenance burden.
            </p>
          </div>
        </div>
      </div>
      <div className="sm:py-20 lg:py-24">
        <Container>
          <div className="px-4 py-16 mx-auto mt-10 sm:max-w-none sm:flex sm:justify-center">
            <div className="space-y-4 sm:space-y-0 sm:mx-auto ">
              <Link href="/docs/getting-started">
                <a className="flex items-center justify-center w-full px-8 py-3 text-base font-medium text-white no-underline bg-black border border-transparent rounded-md dark:bg-white dark:text-black betterhover:dark:hover:bg-gray-300 betterhover:hover:bg-gray-700 md:py-3 md:text-lg md:px-10 md:leading-6">
                  Start Building →
                </a>
              </Link>
            </div>
          </div>
        </Container>
      </div>
      <Toaster position="bottom-right" />
    </>
  );
}
