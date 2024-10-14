# `eslint-plugin-titan`

Ease configuration for Titanrepo

## Installation

1. You'll first need to install [ESLint](https://eslint.org/):

```sh
npm install eslint --save-dev
```

2. Next, install `eslint-plugin-titan`:

```sh
npm install eslint-plugin-titan --save-dev
```

## Usage

Add `titan` to the plugins section of your `.eslintrc` configuration file. You can omit the `eslint-plugin-` prefix:

```json
{
  "plugins": ["titan"]
}
```

Then configure the rules you want to use under the rules section.

```json
{
  "rules": {
    "titan/no-undeclared-env-vars": "error"
  }
}
```

### Example

```json
{
  "plugins": ["titan"],
  "rules": {
    "titan/no-undeclared-env-vars": [
      "error",
      {
        "allowList": ["^ENV_[A-Z]+$"]
      }
    ]
  }
}
```
