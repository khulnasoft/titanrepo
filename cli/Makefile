TITAN_VERSION = $(shell cat ../version.txt | sed -n '1 p')
TITAN_TAG = $(shell cat ../version.txt | sed -n '2 p')

ifeq ($(OS),Windows_NT)
	UNAME := Windows
else
	UNAME := $(shell uname -s)
endif

# Strip debug info
GO_FLAGS += "-ldflags=-s -w"

# Avoid embedding the build path in the executable for more reproducible builds
GO_FLAGS += -trimpath

CLI_DIR = $(shell pwd)

GO_FILES = $(shell find . -name "*.go")
SRC_FILES = $(shell find . -name "*.go" | grep -v "_test.go")
GENERATED_FILES = internal/titandprotocol/titand.pb.go internal/titandprotocol/titand_grpc.pb.go

titan: $(GENERATED_FILES) $(SRC_FILES) go.mod
	CGO_ENABLED=1 go build $(GO_FLAGS) ./cmd/titan

shim: libtitan.a
	cargo build --manifest-path ../shim/titanrepo/Cargo.toml

libtitan.a: $(GENERATED_FILES) $(SRC_FILES) go.mod
	go build -buildmode=c-archive -o libtitan.a  ./cmd/titan/...

protoc: internal/titandprotocol/titand.proto
	protoc --go_out=. --go_opt=paths=source_relative \
		--go-grpc_out=. --go-grpc_opt=paths=source_relative \
		internal/titandprotocol/titand.proto

$(GENERATED_FILES): internal/titandprotocol/titand.proto
	make protoc

compile-protos: $(GENERATED_FILES)

ewatch: scripts/...
	nodemon --exec "make e2e" -e .ts,.go

check-go-version:
	@go version | grep ' go1\.18\.0 ' || (echo 'Please install Go version 1.18.0' && false)

# This "TITAN_RACE" variable exists at the request of a user on GitHub who
# wants to run "make test-go" on an unsupported version of macOS (version 10.9).
# Go's race detector does not run correctly on that version. With this flag
# you can run "TITAN_RACE= make test-go" to disable the race detector.
TITAN_RACE ?= -race

ifeq ($(UNAME), Windows)
	TITAN_RACE=
endif

clean-go:
	go clean -testcache ./...

test-go: $(GENERATED_FILES) $(GO_FILES) go.mod go.sum
	go test $(TITAN_RACE) ./...

# protos need to be compiled before linting, since linting needs to pick up
# some types from the generated code
lint-go: $(GENERATED_FILES) $(GO_FILES) go.mod go.sum
	golangci-lint run --fix

fmt-go: $(GO_FILES) go.mod go.sum
	go fmt ./...

install: | ./package.json
	pnpm install --filter=cli

corepack:
	which corepack || npm install -g corepack@latest
	corepack enable

e2e: corepack install titan
	node -r esbuild-register scripts/e2e/e2e.ts

cmd/titan/version.go: ../version.txt
	# Update this atomically to avoid issues with this being overwritten during use
	node -e 'console.log(`package main\n\nconst titanVersion = "$(TITAN_VERSION)"`)' > cmd/titan/version.go.txt
	mv cmd/titan/version.go.txt cmd/titan/version.go

build: install
	# cd $(CLI_DIR)/../packages/titan-ignore && pnpm install --filter=titan-ignore && npm run build
	cd $(CLI_DIR)/../packages/create-titan && pnpm install --filter=create-titan && npm run build
	cd $(CLI_DIR)/../packages/titan-codemod && pnpm install --filter=@titan/codemod && npm run build

.PHONY: prepublish
prepublish: compile-protos cmd/titan/version.go
	make -j3 bench/titan test-go

.PHONY: publish-titan-cross
publish-titan-cross: prepublish
	goreleaser release --rm-dist -f cross-release.yml

.PHONY: publish-titan-darwin
publish-titan-darwin: prepublish
	goreleaser release --rm-dist -f darwin-release.yml

.PHONY: snapshot-titan-cross
snapshot-titan-cross:
	goreleaser release --snapshot --rm-dist -f cross-release.yml

.PHONY: snapshot-titan-darwin
snapshot-titan-darwin:
	goreleaser release --snapshot --rm-dist -f darwin-release.yml

.PHONY: snapshot-titan
snapshot-titan: clean
	goreleaser release --snapshot --rm-dist -f combined-release.yml

.PHONY: snapshot-lib-titan-darwin
snapshot-lib-titan-darwin:
	goreleaser release --snapshot --rm-dist -f darwin-lib.yml

.PHONY: snapshot-lib-titan-cross
snapshot-lib-titan-cross:
	goreleaser release --snapshot --rm-dist -f cross-lib.yml

.PHONY: build-lib-titan-darwin
build-lib-titan-darwin:
	goreleaser release --rm-dist -f darwin-lib.yml

.PHONY: build-lib-titan-cross
build-lib-titan-cross:
	goreleaser release --rm-dist -f cross-lib.yml


.PHONY: stage-release
stage-release: cmd/titan/version.go
	echo "Version: $(TITAN_VERSION)"
	echo "Tag: $(TITAN_TAG)"
	cat $(CLI_DIR)/../version.txt
	git diff -- $(CLI_DIR)/../version.txt
	git status
	@test main = "`git rev-parse --abbrev-ref HEAD`" || (echo "Refusing to publish from non-main branch `git rev-parse --abbrev-ref HEAD`" && false)
	@test "" = "`git cherry`" || (echo "Refusing to publish with unpushed commits" && false)

	# Stop if versions are not updated.
	@test "" != "`git diff -- $(CLI_DIR)/../version.txt`" || (echo "Refusing to publish with unupdated version.txt" && false)
	@test "" != "`git diff -- $(CLI_DIR)/cmd/titan/version.go`" || (echo "Refusing to publish with unupdated version.go" && false)

	# Prepare the packages.
	cd $(CLI_DIR)/../packages/titan && npm version "$(TITAN_VERSION)" --allow-same-version
	# cd $(CLI_DIR)/../packages/titan-ignore && npm version "$(TITAN_VERSION)" --allow-same-version
	cd $(CLI_DIR)/../packages/create-titan && npm version "$(TITAN_VERSION)" --allow-same-version
	cd $(CLI_DIR)/../packages/titan-codemod && npm version "$(TITAN_VERSION)" --allow-same-version

	git checkout -b staging-$(TITAN_VERSION)
	git commit -anm "publish $(TITAN_VERSION) to registry"
	git tag "v$(TITAN_VERSION)"
	git push origin staging-$(TITAN_VERSION) --tags --force

.PHONY: publish
publish: clean build
	echo "Version: $(TITAN_VERSION)"
	echo "Tag: $(TITAN_TAG)"

	# Include the patch in the log.
	git format-patch HEAD~1 --stdout | cat

	npm config set --location=project "//registry.npmjs.org/:_authToken" $(NPM_TOKEN)


	# Publishes the native npm modules.
	goreleaser release --rm-dist -f combined-release.yml

	# Split packing from the publish step so that npm locates the correct .npmrc file.
	npm pack $(CLI_DIR)/../packages/titan --pack-destination=$(CLI_DIR)/../
	npm pack $(CLI_DIR)/../packages/titan-ignore --pack-destination=$(CLI_DIR)/../
	npm pack $(CLI_DIR)/../packages/create-titan --pack-destination=$(CLI_DIR)/../
	npm pack $(CLI_DIR)/../packages/titan-codemod --pack-destination=$(CLI_DIR)/../

	# Publish the remaining JS packages in order to avoid race conditions.
	cd $(CLI_DIR)/../
	npm publish -ddd --tag $(TITAN_TAG) $(CLI_DIR)/../titan-$(TITAN_VERSION).tgz
	# npm publish -ddd --tag $(TITAN_TAG) $(CLI_DIR)/../titan-ignore-$(TITAN_VERSION).tgz
	npm publish -ddd --tag $(TITAN_TAG) $(CLI_DIR)/../create-titan-$(TITAN_VERSION).tgz
	npm publish -ddd --tag $(TITAN_TAG) $(CLI_DIR)/../titan-codemod-$(TITAN_VERSION).tgz

.PHONY: snapshot-shim
snapshot-shim:
	echo "Version: $(TITAN_VERSION)"
	echo "Tag: $(TITAN_TAG)"

	# Include the patch in the log.
	git format-patch HEAD~1 --stdout | cat

	npm config set --location=project "//registry.npmjs.org/:_authToken" $(NPM_TOKEN)

	# Publishes the native npm modules.
	goreleaser release --rm-dist -f combined-shim.yml --snapshot

	# Split packing from the publish step so that npm locates the correct .npmrc file.
	npm pack $(CLI_DIR)/../packages/titan --pack-destination=$(CLI_DIR)/../
	npm pack $(CLI_DIR)/../packages/titan-ignore --pack-destination=$(CLI_DIR)/../
	npm pack $(CLI_DIR)/../packages/create-titan --pack-destination=$(CLI_DIR)/../
	npm pack $(CLI_DIR)/../packages/titan-codemod --pack-destination=$(CLI_DIR)/../


.PHONY: publish-shim
publish-shim:
	echo "Version: $(TITAN_VERSION)"
	echo "Tag: $(TITAN_TAG)"

	# Include the patch in the log.
	git format-patch HEAD~1 --stdout | cat

	npm config set --location=project "//registry.npmjs.org/:_authToken" $(NPM_TOKEN)

	# Publishes the native npm modules.
	goreleaser release --rm-dist -f combined-shim.yml

	# Split packing from the publish step so that npm locates the correct .npmrc file.
	npm pack $(CLI_DIR)/../packages/titan --pack-destination=$(CLI_DIR)/../
	npm pack $(CLI_DIR)/../packages/titan-ignore --pack-destination=$(CLI_DIR)/../
	npm pack $(CLI_DIR)/../packages/create-titan --pack-destination=$(CLI_DIR)/../
	npm pack $(CLI_DIR)/../packages/titan-codemod --pack-destination=$(CLI_DIR)/../

	# Publish the remaining JS packages in order to avoid race conditions.
	cd $(CLI_DIR)/../
	npm publish -ddd --tag $(TITAN_TAG) $(CLI_DIR)/../titan-$(TITAN_VERSION).tgz
	# npm publish -ddd --tag $(TITAN_TAG) $(CLI_DIR)/../titan-ignore-$(TITAN_VERSION).tgz
	npm publish -ddd --tag $(TITAN_TAG) $(CLI_DIR)/../create-titan-$(TITAN_VERSION).tgz
	npm publish -ddd --tag $(TITAN_TAG) $(CLI_DIR)/../titan-codemod-$(TITAN_VERSION).tgz

demo/lage: install
	node $(CLI_DIR)/scripts/generate.mjs lage

demo/lerna: install
	node $(CLI_DIR)/scripts/generate.mjs lerna

demo/nx: install
	node $(CLI_DIR)/scripts/generate.mjs nx

demo/titan: install
	node $(CLI_DIR)/scripts/generate.mjs titan

demo: demo/lage demo/lerna demo/nx demo/titan

bench/lerna: demo/lerna
	cd $(CLI_DIR)/demo/lerna && node_modules/.bin/lerna run build

bench/lage: demo/lage
	cd $(CLI_DIR)/demo/lage && node_modules/.bin/lage build

bench/nx: demo/nx
	cd $(CLI_DIR)/demo/nx && node_modules/.bin/nx run-many --target=build --all

bench/titan: demo/titan titan
	cd $(CLI_DIR)/demo/titan && $(CLI_DIR)/titan run test

bench: bench/lerna bench/lage bench/nx bench/titan

clean: clean-go clean-build clean-demo

clean-build:
	rm -f titan

clean-demo:
	rm -rf node_modules
	rm -rf demo

CRAM_ENV := .cram_env

$(CRAM_ENV)/bin/pip:
	python3 -m venv $(CRAM_ENV)

$(CRAM_ENV)/bin/prysk: $(CRAM_ENV)/bin/pip
	$(CRAM_ENV)/bin/pip install prysk

INTEGRATION_TEST_FILES = $(shell find integration_tests -name "*.t")

integration-tests: $(CRAM_ENV)/bin/prysk titan $(INTEGRATION_TEST_FILES) corepack
	$(CRAM_ENV)/bin/prysk --shell=`which bash` $(INTEGRATION_TEST_FILES)

# use target testbed-<some directory under integration_tests> to set up the testbed directory
.PHONY=testbed-%
testbed-%:
	$(eval $@_TEST := $(@:testbed-%=%))
	@echo "testbed setup $($@_TEST)"
	rm -rf testbed
	mkdir -p testbed
	./integration_tests/$($@_TEST)/setup.sh testbed

