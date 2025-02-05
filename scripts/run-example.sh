#!/bin/bash

set -e

echo "=> Running examples..."
# echo "=> Building titan from source..."
# cd cli && CGO_ENABLED=0 go build ./cmd/titan/... && cd ..;
export TITAN_BINARY_PATH=$(pwd)/cli/titan
export TITAN_VERSION=$(head -n 1 $(pwd)/version.txt)
export TITAN_TAG=$(cat $(pwd)/version.txt | sed -n '2 p')
export folder=$1
export pkgManager=$2
echo "=> Binary path: TITAN_BINARY_PATH=$TITAN_BINARY_PATH"
echo "=> Local Turbo Version: TITAN_VERSION=$TITAN_VERSION"
echo "=> Moving our own eslint settings out of the way..."
echo "=> Actually running examples for real..."

if [ -f ".eslintrc.js" ]; then
  mv .eslintrc.js .eslintrc.js.bak
fi

function cleanup {
  rm -rf node_modules
  rm -rf apps/*/node_modules
  rm -rf apps/*/.next
  rm -rf apps/*/.titan
  rm -rf packages/*/node_modules
  rm -rf packages/*/.next
  rm -rf packages/*/.titan
  rm -rf *.log
  rm -rf yarn.lock
  rm -rf package-lock.json
  rm -rf pnpm-lock.yaml
}

function setup_git {
  echo "=> Setting up git..."
  rm -rf .git
  mkdir .git
  touch .git/config
  echo "[user]" >> .git/config
  echo "  name = GitHub Actions" >> .git/config
  echo "  email = actions@users.noreply.github.com" >> .git/config
  echo "" >> .git/config
  echo "[init]" >> .git/config
  echo "  defaultBranch = main" >> .git/config
  git init . -q
  git add .
  git commit -m "Initial commit"
}

function run_npm {
  cat package.json | jq '.packageManager = "npm@8.1.2"' | sponge package.json
  if [ "$TITAN_TAG" == "canary" ]; then
    cat package.json | jq '.devDependencies.titan = "canary"' | sponge package.json
  fi

  echo "======================================================="
  echo "=> $folder: npm install"
  echo "======================================================="
  npm install --force

  echo "======================================================="
  echo "=> $folder: npm build lint"
  echo "======================================================="
  npm run build lint

  echo "======================================================="
  echo "=> $folder: npm build lint again"
  echo "======================================================="
  npm run build lint

  echo "======================================================="
  echo "=> $folder: npm SUCCESSFUL"
  echo "======================================================="
}

function run_pnpm {
  cat package.json | jq '.packageManager = "pnpm@6.26.1"' | sponge package.json
  if [ "$TITAN_TAG" == "canary" ]; then
    cat package.json | jq '.devDependencies.titan = "canary"' | sponge package.json
  fi

  echo "======================================================="
  echo "=> $folder: pnpm install"
  echo "======================================================="
  pnpm install

  echo "======================================================="
  echo "=> $folder: pnpm build lint"
  echo "======================================================="
  pnpm run build lint

  echo "======================================================="
  echo "=> $folder: pnpm build lint again"
  echo "======================================================="
  pnpm run build lint

  echo "======================================================="
  echo "=> $folder: pnpm SUCCESSFUL"
  echo "======================================================="
}

function run_yarn {
  cat package.json | jq '.packageManager = "yarn@1.22.17"' | sponge package.json
  if [ "$TITAN_TAG" == "canary" ]; then
    cat package.json | jq '.devDependencies.titan = "canary"' | sponge package.json
  fi

  echo "======================================================="
  echo "=> $folder: yarn install"
  echo "======================================================="
  yarn install

  echo "======================================================="
  echo "=> $folder: yarn build lint"
  echo "======================================================="
  yarn build lint

  echo "======================================================="
  echo "=> $folder: yarn build again"
  echo "======================================================="
  yarn build lint

  echo "======================================================="
  echo "=> $folder: yarn SUCCESSFUL"
  echo "======================================================="
}


hasRun=0
if [ -f "examples/$folder/package.json" ]; then
  cd "examples/$folder"
  echo "======================================================="
  echo "=> checking $folder "
  echo "======================================================="
  cleanup
  setup_git
  if [ "$pkgManager" == "npm" ]; then
    run_npm
  elif [ "$pkgManager" == "pnpm" ]; then
    run_pnpm
  elif [ "$pkgManager" == "yarn" ]; then
    run_yarn
  else
    echo "Unknown package manager ${folder}"
    exit 2
  fi
  hasRun=1

  cat package.json | jq 'del(.packageManager)' | sponge package.json
  if [ "$TITAN_TAG" == "canary" ]; then
    cat package.json | jq '.devDependencies.titan = "latest"' | sponge package.json
  fi

  cleanup

  cd ../..
fi

if [ -f ".eslintrc.js.bak" ]; then
  mv .eslintrc.js.bak .eslintrc.js
fi

if [[ ! -z $(git status -s) ]];then
  echo "Detected changes"
  git status
  exit 1
fi

if [ $hasRun -eq 0 ]; then
  echo "Did not run any examples"
  exit 2
fi
