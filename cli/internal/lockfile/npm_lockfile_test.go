package lockfile

import (
	"sort"
	"testing"

	"github.com/khulnasoft/titanrepo/cli/internal/titanpath"
	"gotest.tools/v3/assert"
	"gotest.tools/v3/assert/cmp"
)

func getNpmLockfile(t *testing.T) *NpmLockfile {
	content, err := getFixture(t, "npm-lock.json")
	assert.NilError(t, err, "reading npm-lock.json")
	lockfile, err := DecodeNpmLockfile(content)
	assert.NilError(t, err, "parsing npm-lock.json")
	return lockfile
}

func Test_NpmPathParent(t *testing.T) {
	type TestCase struct {
		key    string
		parent string
	}
	testCases := []TestCase{
		{
			key:    "apps/docs",
			parent: "",
		},
		{
			key:    "apps/docs/node_modules/foo",
			parent: "apps/docs/",
		},
		{
			key:    "node_modules/foo",
			parent: "",
		},
		{
			key:    "node_modules/foo/node_modules/bar",
			parent: "node_modules/foo/",
		},
	}

	for _, tc := range testCases {
		assert.Equal(t, npmPathParent(tc.key), tc.parent, tc.key)
	}
}

func Test_PossibleNpmDeps(t *testing.T) {
	type TestCase struct {
		name     string
		key      string
		dep      string
		expected []string
	}
	testCases := []TestCase{
		{
			name: "top level looks for children",
			key:  "node_modules/foo",
			dep:  "baz",
			expected: []string{
				"node_modules/foo/node_modules/baz",
				"node_modules/baz",
			},
		},
		{
			name: "if child looks for siblings",
			key:  "node_modules/foo/node_modules/bar",
			dep:  "baz",
			expected: []string{
				"node_modules/foo/node_modules/bar/node_modules/baz",
				"node_modules/foo/node_modules/baz",
				"node_modules/baz",
			},
		},
		{
			name: "deeply nested package looks through all ancestors",
			key:  "node_modules/foo1/node_modules/foo2/node_modules/foo3/node_modules/foo4",
			dep:  "bar",
			expected: []string{
				"node_modules/foo1/node_modules/foo2/node_modules/foo3/node_modules/foo4/node_modules/bar",
				"node_modules/foo1/node_modules/foo2/node_modules/foo3/node_modules/bar",
				"node_modules/foo1/node_modules/foo2/node_modules/bar",
				"node_modules/foo1/node_modules/bar",
				"node_modules/bar",
			},
		},
		{
			name: "workspace deps look for nested",
			key:  "apps/docs/node_modules/foo",
			dep:  "baz",
			expected: []string{
				"apps/docs/node_modules/foo/node_modules/baz",
				"apps/docs/node_modules/baz",
				"node_modules/baz",
			},
		},
	}

	for _, tc := range testCases {
		actual := possibleNpmDeps(tc.key, tc.dep)
		assert.Assert(t, cmp.DeepEqual(actual, tc.expected), tc.name)
	}
}

func Test_NpmResolvePackage(t *testing.T) {
	type TestCase struct {
		testName  string
		workspace string
		name      string
		key       string
		version   string
		err       string
	}
	testCases := []TestCase{
		{
			testName:  "finds deps of root package",
			workspace: "",
			name:      "titan",
			key:       "node_modules/titan",
			version:   "1.5.5",
		},
		{
			testName:  "selects nested dep if present",
			workspace: "apps/web",
			name:      "lodash",
			key:       "apps/web/node_modules/lodash",
			version:   "4.17.21",
		},
		{
			testName:  "selects top level package if no nested package",
			workspace: "apps/docs",
			name:      "lodash",
			key:       "node_modules/lodash",
			version:   "3.10.1",
		},
		{
			testName:  "finds package if given resolved key",
			workspace: "apps/docs",
			name:      "node_modules/@babel/generator/node_modules/@jridgewell/gen-mapping",
			key:       "node_modules/@babel/generator/node_modules/@jridgewell/gen-mapping",
			version:   "0.3.2",
		},
	}

	lockfile := getNpmLockfile(t)
	for _, tc := range testCases {
		workspace := titanpath.AnchoredUnixPath(tc.workspace)
		pkg, err := lockfile.ResolvePackage(workspace, tc.name, "")
		if tc.err != "" {
			assert.Error(t, err, tc.err)
		} else {
			assert.NilError(t, err, tc.testName)
		}
		assert.Assert(t, pkg.Found, tc.testName)
		assert.Equal(t, pkg.Key, tc.key, tc.testName)
		assert.Equal(t, pkg.Version, tc.version, tc.testName)
	}
}

func Test_NpmAllDependencies(t *testing.T) {
	type TestCase struct {
		name     string
		key      string
		expected []string
	}
	testCases := []TestCase{
		{
			name: "mixed nested and hoisted",
			key:  "node_modules/table",
			expected: []string{
				"node_modules/lodash.truncate",
				"node_modules/slice-ansi",
				"node_modules/string-width",
				"node_modules/strip-ansi",
				"node_modules/table/node_modules/ajv",
			},
		},
		{
			name: "deps of nested packaged",
			key:  "node_modules/table/node_modules/ajv",
			expected: []string{
				"node_modules/fast-deep-equal",
				"node_modules/require-from-string",
				"node_modules/table/node_modules/json-schema-traverse",
				"node_modules/uri-js",
			},
		},
	}

	lockfile := getNpmLockfile(t)
	for _, tc := range testCases {
		deps, ok := lockfile.AllDependencies(tc.key)
		assert.Assert(t, ok, tc.name)
		depKeys := make([]string, len(deps))
		i := 0
		for dep := range deps {
			depKeys[i] = dep
			i++
		}
		sort.Strings(depKeys)
		assert.DeepEqual(t, depKeys, tc.expected)
	}

}
