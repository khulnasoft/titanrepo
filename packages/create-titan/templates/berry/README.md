# Titanrepo starter

This is an official Yarn (Berry) starter titanrepo.

## What's inside?

This titanrepo uses [Yarn](https://yarnpkg.com/) as a package manager. It includes the following packages/apps:

### Apps and Packages

- `docs`: a [Next.js](https://nextjs.org) app
- `web`: another [Next.js](https://nextjs.org) app
- `ui`: a stub React component library shared by both `web` and `docs` applications
- `eslint-config-custom`: `eslint` configurations (includes `eslint-config-next` and `eslint-config-prettier`)
- `tsconfig`: `tsconfig.json`s used throughout the monorepo

Each package/app is 100% [TypeScript](https://www.typescriptlang.org/).

### Utilities

This titanrepo has some additional tools already setup for you:

- [TypeScript](https://www.typescriptlang.org/) for static type checking
- [ESLint](https://eslint.org/) for code linting
- [Prettier](https://prettier.io) for code formatting

### Build

To build all apps and packages, run the following command:

```
cd my-titanrepo
yarn run build
```

### Develop

To develop all apps and packages, run the following command:

```
cd my-titanrepo
yarn run dev
```

### Remote Caching

Titanrepo can use a technique known as [Remote Caching](https://titan.khulnasoft.com/docs/core-concepts/remote-caching) to share cache artifacts across machines, enabling you to share build caches with your team and CI/CD pipelines.

By default, Titanrepo will cache locally. To enable Remote Caching you will need an account with Khulnasoft. If you don't have an account you can [create one](https://khulnasoft.com/signup), then enter the following commands:

```
cd my-titanrepo
yarn dlx titan login
```

This will authenticate the Titanrepo CLI with your [Khulnasoft account](https://khulnasoft.com/docs/concepts/personal-accounts/overview).

Next, you can link your Titanrepo to your Remote Cache by running the following command from the root of your titanrepo:

```
yarn dlx titan link
```

## Useful Links

Learn more about the power of Titanrepo:

- [Pipelines](https://titan.khulnasoft.com/docs/core-concepts/pipelines)
- [Caching](https://titan.khulnasoft.com/docs/core-concepts/caching)
- [Remote Caching](https://titan.khulnasoft.com/docs/core-concepts/remote-caching)
- [Scoped Tasks](https://titan.khulnasoft.com/docs/core-concepts/scopes)
- [Configuration Options](https://titan.khulnasoft.com/docs/reference/configuration)
- [CLI Usage](https://titan.khulnasoft.com/docs/reference/command-line-reference)
