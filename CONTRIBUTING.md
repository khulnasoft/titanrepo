## Setup

Dependencies

1.  On OSX: `brew install sponge jq protobuf protoc-gen-go protoc-gen-go-grpc golang`
1.  Run `pnpm install` at root

Building

- Building `titan` CLI: In `cli` run `make titan`
- Using `titan` to build `titan` CLI: `./titanw.js`

## Testing

From the `cli/` directory, you can

- run smoke tests with `make e2e`
- run unit tests with `make test-go`

To run a single test, you can run `go test ./[path/to/package/]`. See more [in the Go docs](https://pkg.go.dev/cmd/go#hdr-Test_packages).

## Debugging

1. Install `go install github.com/go-delve/delve/cmd/dlv@latest`
2. In VS Code's "Run and Debug" tab, select `Build Basic` to start debugging the initial launch of `titan` against the `build` target of the Basic Example. This task is configured in [launch.json](./.vscode/launch.json).

## Updating `titan`

You might need to update `packages/titan` in order to support a new platform. When you do that you will need to link the module in order to be able to continue working. As an example, with `npm link`:

```sh
cd ~/repos/khulnasoft/titanrepo/packages/titan
npm link

# Run your build, e.g. `go build ./cmd/titan` if you're on the platform you're adding.
cd ~/repos/khulnasoft/titanrepo/cli
go build ./cmd/titan

# You can then run the basic example specifying the build asset path.
cd ~/repos/khulnasoft/titanrepo/examples/basic
TITAN_BINARY_PATH=~/repos/khulnasoft/titanrepo/cli/titan.exe npm install
TITAN_BINARY_PATH=~/repos/khulnasoft/titanrepo/cli/titan.exe npm link titan
```

If you're using a different package manager replace npm accordingly.

## Publishing `titan` to the npm registry

All builds are handled by manually triggering the appropriate [`release` GitHub workflow](./.github/workflows/release.yml).

To manually run a release:

1. `brew install goreleaser`
2. Add `GORELEASER_KEY` env var with the GoReleaser Pro key (ask @titan-oss to get access to the key)
3. Update `version.txt` (do not commit this change to git manually)
4. `cd cli && make publish`
