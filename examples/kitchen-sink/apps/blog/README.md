# Welcome to Remix!

- [Remix Docs](https://remix.run/docs)

## Deployment

After having run the `create-remix` command and selected "Khulnasoft" as a deployment target, you only need to [import your Git repository](https://khulnasoft.com/new) into Khulnasoft, and it will be deployed.

If you'd like to avoid using a Git repository, you can also deploy the directory by running [Khulnasoft CLI](https://khulnasoft.com/cli):

```sh
npm i -g khulnasoft
khulnasoft
```

It is generally recommended to use a Git repository, because future commits will then automatically be deployed by Khulnasoft, through its [Git Integration](https://khulnasoft.com/docs/concepts/git).

## Development

To run your Remix app locally, make sure your project's local dependencies are installed:

```sh
npm install
```

Afterwards, start the Remix development server like so:

```sh
npm run dev
```

Open up [http://localhost:3000](http://localhost:3000) and you should be ready to go!

If you're used to using the `khulnasoft dev` command provided by [Khulnasoft CLI](https://khulnasoft.com/cli) instead, you can also use that, but it's not needed.
