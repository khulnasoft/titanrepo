package packagemanager

import (
	"reflect"
	"testing"

	"github.com/khulnasoft/titanrepo/cli/internal/titanpath"
	"gotest.tools/v3/assert"
)

func TestInferRoot(t *testing.T) {
	type file struct {
		path    titanpath.AnchoredSystemPath
		content []byte
	}

	tests := []struct {
		name               string
		fs                 []file
		executionDirectory titanpath.AnchoredSystemPath
		rootPath           titanpath.AnchoredSystemPath
		packageMode        PackageType
	}{
		// Scenario 0
		{
			name: "titan.json at current dir, no package.json",
			fs: []file{
				{path: titanpath.AnchoredUnixPath("titan.json").ToSystemPath()},
			},
			executionDirectory: titanpath.AnchoredUnixPath("").ToSystemPath(),
			rootPath:           titanpath.AnchoredUnixPath("").ToSystemPath(),
			packageMode:        Multi,
		},
		{
			name: "titan.json at parent dir, no package.json",
			fs: []file{
				{path: titanpath.AnchoredUnixPath("execution/path/subdir/.file").ToSystemPath()},
				{path: titanpath.AnchoredUnixPath("titan.json").ToSystemPath()},
			},
			executionDirectory: titanpath.AnchoredUnixPath("execution/path/subdir").ToSystemPath(),
			// This is "no inference"
			rootPath:    titanpath.AnchoredUnixPath("execution/path/subdir").ToSystemPath(),
			packageMode: Multi,
		},
		// Scenario 1A
		{
			name: "titan.json at current dir, has package.json, has workspaces key",
			fs: []file{
				{path: titanpath.AnchoredUnixPath("titan.json").ToSystemPath()},
				{
					path:    titanpath.AnchoredUnixPath("package.json").ToSystemPath(),
					content: []byte("{ \"workspaces\": [ \"exists\" ] }"),
				},
			},
			executionDirectory: titanpath.AnchoredUnixPath("").ToSystemPath(),
			rootPath:           titanpath.AnchoredUnixPath("").ToSystemPath(),
			packageMode:        Multi,
		},
		{
			name: "titan.json at parent dir, has package.json, has workspaces key",
			fs: []file{
				{path: titanpath.AnchoredUnixPath("execution/path/subdir/.file").ToSystemPath()},
				{path: titanpath.AnchoredUnixPath("titan.json").ToSystemPath()},
				{
					path:    titanpath.AnchoredUnixPath("package.json").ToSystemPath(),
					content: []byte("{ \"workspaces\": [ \"exists\" ] }"),
				},
			},
			executionDirectory: titanpath.AnchoredUnixPath("execution/path/subdir").ToSystemPath(),
			rootPath:           titanpath.AnchoredUnixPath("").ToSystemPath(),
			packageMode:        Multi,
		},
		{
			name: "titan.json at parent dir, has package.json, has pnpm workspaces",
			fs: []file{
				{path: titanpath.AnchoredUnixPath("execution/path/subdir/.file").ToSystemPath()},
				{path: titanpath.AnchoredUnixPath("titan.json").ToSystemPath()},
				{
					path:    titanpath.AnchoredUnixPath("package.json").ToSystemPath(),
					content: []byte("{}"),
				},
				{
					path:    titanpath.AnchoredUnixPath("pnpm-workspace.yaml").ToSystemPath(),
					content: []byte("packages:\n  - docs"),
				},
			},
			executionDirectory: titanpath.AnchoredUnixPath("execution/path/subdir").ToSystemPath(),
			rootPath:           titanpath.AnchoredUnixPath("").ToSystemPath(),
			packageMode:        Multi,
		},
		// Scenario 1A aware of the weird thing we do for packages.
		{
			name: "titan.json at current dir, has package.json, has packages key",
			fs: []file{
				{path: titanpath.AnchoredUnixPath("titan.json").ToSystemPath()},
				{
					path:    titanpath.AnchoredUnixPath("package.json").ToSystemPath(),
					content: []byte("{ \"packages\": [ \"exists\" ] }"),
				},
			},
			executionDirectory: titanpath.AnchoredUnixPath("").ToSystemPath(),
			rootPath:           titanpath.AnchoredUnixPath("").ToSystemPath(),
			packageMode:        Single,
		},
		{
			name: "titan.json at parent dir, has package.json, has packages key",
			fs: []file{
				{path: titanpath.AnchoredUnixPath("execution/path/subdir/.file").ToSystemPath()},
				{path: titanpath.AnchoredUnixPath("titan.json").ToSystemPath()},
				{
					path:    titanpath.AnchoredUnixPath("package.json").ToSystemPath(),
					content: []byte("{ \"packages\": [ \"exists\" ] }"),
				},
			},
			executionDirectory: titanpath.AnchoredUnixPath("execution/path/subdir").ToSystemPath(),
			rootPath:           titanpath.AnchoredUnixPath("").ToSystemPath(),
			packageMode:        Single,
		},
		// Scenario 1A aware of the the weird thing we do for packages when both methods of specification exist.
		{
			name: "titan.json at current dir, has package.json, has workspace and packages key",
			fs: []file{
				{path: titanpath.AnchoredUnixPath("titan.json").ToSystemPath()},
				{
					path:    titanpath.AnchoredUnixPath("package.json").ToSystemPath(),
					content: []byte("{ \"workspaces\": [ \"clobbered\" ], \"packages\": [ \"exists\" ] }"),
				},
			},
			executionDirectory: titanpath.AnchoredUnixPath("").ToSystemPath(),
			rootPath:           titanpath.AnchoredUnixPath("").ToSystemPath(),
			packageMode:        Multi,
		},
		{
			name: "titan.json at parent dir, has package.json, has workspace and packages key",
			fs: []file{
				{path: titanpath.AnchoredUnixPath("execution/path/subdir/.file").ToSystemPath()},
				{path: titanpath.AnchoredUnixPath("titan.json").ToSystemPath()},
				{
					path:    titanpath.AnchoredUnixPath("package.json").ToSystemPath(),
					content: []byte("{ \"workspaces\": [ \"clobbered\" ], \"packages\": [ \"exists\" ] }"),
				},
			},
			executionDirectory: titanpath.AnchoredUnixPath("execution/path/subdir").ToSystemPath(),
			rootPath:           titanpath.AnchoredUnixPath("").ToSystemPath(),
			packageMode:        Multi,
		},
		// Scenario 1B
		{
			name: "titan.json at current dir, has package.json, no workspaces",
			fs: []file{
				{path: titanpath.AnchoredUnixPath("titan.json").ToSystemPath()},
				{
					path:    titanpath.AnchoredUnixPath("package.json").ToSystemPath(),
					content: []byte("{}"),
				},
			},
			executionDirectory: titanpath.AnchoredUnixPath("").ToSystemPath(),
			rootPath:           titanpath.AnchoredUnixPath("").ToSystemPath(),
			packageMode:        Single,
		},
		{
			name: "titan.json at parent dir, has package.json, no workspaces",
			fs: []file{
				{path: titanpath.AnchoredUnixPath("execution/path/subdir/.file").ToSystemPath()},
				{path: titanpath.AnchoredUnixPath("titan.json").ToSystemPath()},
				{
					path:    titanpath.AnchoredUnixPath("package.json").ToSystemPath(),
					content: []byte("{}"),
				},
			},
			executionDirectory: titanpath.AnchoredUnixPath("execution/path/subdir").ToSystemPath(),
			rootPath:           titanpath.AnchoredUnixPath("").ToSystemPath(),
			packageMode:        Single,
		},
		{
			name: "titan.json at parent dir, has package.json, no workspaces, includes pnpm",
			fs: []file{
				{path: titanpath.AnchoredUnixPath("execution/path/subdir/.file").ToSystemPath()},
				{path: titanpath.AnchoredUnixPath("titan.json").ToSystemPath()},
				{
					path:    titanpath.AnchoredUnixPath("package.json").ToSystemPath(),
					content: []byte("{}"),
				},
				{
					path:    titanpath.AnchoredUnixPath("pnpm-workspace.yaml").ToSystemPath(),
					content: []byte(""),
				},
			},
			executionDirectory: titanpath.AnchoredUnixPath("execution/path/subdir").ToSystemPath(),
			rootPath:           titanpath.AnchoredUnixPath("").ToSystemPath(),
			packageMode:        Single,
		},
		// Scenario 2A
		{
			name:               "no titan.json, no package.json at current",
			fs:                 []file{},
			executionDirectory: titanpath.AnchoredUnixPath("").ToSystemPath(),
			rootPath:           titanpath.AnchoredUnixPath("").ToSystemPath(),
			packageMode:        Multi,
		},
		{
			name: "no titan.json, no package.json at parent",
			fs: []file{
				{path: titanpath.AnchoredUnixPath("execution/path/subdir/.file").ToSystemPath()},
			},
			executionDirectory: titanpath.AnchoredUnixPath("execution/path/subdir").ToSystemPath(),
			rootPath:           titanpath.AnchoredUnixPath("execution/path/subdir").ToSystemPath(),
			packageMode:        Multi,
		},
		// Scenario 2B
		{
			name: "no titan.json, has package.json with workspaces at current",
			fs: []file{
				{
					path:    titanpath.AnchoredUnixPath("package.json").ToSystemPath(),
					content: []byte("{ \"workspaces\": [ \"exists\" ] }"),
				},
			},
			executionDirectory: titanpath.AnchoredUnixPath("").ToSystemPath(),
			rootPath:           titanpath.AnchoredUnixPath("").ToSystemPath(),
			packageMode:        Multi,
		},
		{
			name: "no titan.json, has package.json with workspaces at parent",
			fs: []file{
				{path: titanpath.AnchoredUnixPath("execution/path/subdir/.file").ToSystemPath()},
				{
					path:    titanpath.AnchoredUnixPath("package.json").ToSystemPath(),
					content: []byte("{ \"workspaces\": [ \"exists\" ] }"),
				},
			},
			executionDirectory: titanpath.AnchoredUnixPath("execution/path/subdir").ToSystemPath(),
			rootPath:           titanpath.AnchoredUnixPath("execution/path/subdir").ToSystemPath(),
			packageMode:        Multi,
		},
		{
			name: "no titan.json, has package.json with pnpm workspaces at parent",
			fs: []file{
				{path: titanpath.AnchoredUnixPath("execution/path/subdir/.file").ToSystemPath()},
				{
					path:    titanpath.AnchoredUnixPath("package.json").ToSystemPath(),
					content: []byte("{ \"workspaces\": [ \"exists\" ] }"),
				},
				{
					path:    titanpath.AnchoredUnixPath("pnpm-workspace.yaml").ToSystemPath(),
					content: []byte("packages:\n  - docs"),
				},
			},
			executionDirectory: titanpath.AnchoredUnixPath("execution/path/subdir").ToSystemPath(),
			rootPath:           titanpath.AnchoredUnixPath("execution/path/subdir").ToSystemPath(),
			packageMode:        Multi,
		},
		// Scenario 3A
		{
			name: "no titan.json, lots of package.json files but no workspaces",
			fs: []file{
				{
					path:    titanpath.AnchoredUnixPath("package.json").ToSystemPath(),
					content: []byte("{}"),
				},
				{
					path:    titanpath.AnchoredUnixPath("one/package.json").ToSystemPath(),
					content: []byte("{}"),
				},
				{
					path:    titanpath.AnchoredUnixPath("one/two/package.json").ToSystemPath(),
					content: []byte("{}"),
				},
				{
					path:    titanpath.AnchoredUnixPath("one/two/three/package.json").ToSystemPath(),
					content: []byte("{}"),
				},
			},
			executionDirectory: titanpath.AnchoredUnixPath("one/two/three").ToSystemPath(),
			rootPath:           titanpath.AnchoredUnixPath("one/two/three").ToSystemPath(),
			packageMode:        Single,
		},
		// Scenario 3BI
		{
			name: "no titan.json, lots of package.json files, and a workspace at the root that matches execution directory",
			fs: []file{
				{
					path:    titanpath.AnchoredUnixPath("package.json").ToSystemPath(),
					content: []byte("{ \"workspaces\": [ \"one/two/three\" ] }"),
				},
				{
					path:    titanpath.AnchoredUnixPath("one/package.json").ToSystemPath(),
					content: []byte("{}"),
				},
				{
					path:    titanpath.AnchoredUnixPath("one/two/package.json").ToSystemPath(),
					content: []byte("{}"),
				},
				{
					path:    titanpath.AnchoredUnixPath("one/two/three/package.json").ToSystemPath(),
					content: []byte("{}"),
				},
			},
			executionDirectory: titanpath.AnchoredUnixPath("one/two/three").ToSystemPath(),
			rootPath:           titanpath.AnchoredUnixPath("one/two/three").ToSystemPath(),
			packageMode:        Multi,
		},
		// Scenario 3BII
		{
			name: "no titan.json, lots of package.json files, and a workspace at the root that matches execution directory",
			fs: []file{
				{
					path:    titanpath.AnchoredUnixPath("package.json").ToSystemPath(),
					content: []byte("{ \"workspaces\": [ \"does-not-exist\" ] }"),
				},
				{
					path:    titanpath.AnchoredUnixPath("one/package.json").ToSystemPath(),
					content: []byte("{}"),
				},
				{
					path:    titanpath.AnchoredUnixPath("one/two/package.json").ToSystemPath(),
					content: []byte("{}"),
				},
				{
					path:    titanpath.AnchoredUnixPath("one/two/three/package.json").ToSystemPath(),
					content: []byte("{}"),
				},
			},
			executionDirectory: titanpath.AnchoredUnixPath("one/two/three").ToSystemPath(),
			rootPath:           titanpath.AnchoredUnixPath("one/two/three").ToSystemPath(),
			packageMode:        Single,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			fsRoot := titanpath.AbsoluteSystemPath(t.TempDir())
			for _, file := range tt.fs {
				path := file.path.RestoreAnchor(fsRoot)
				assert.NilError(t, path.Dir().MkdirAll(0777))
				assert.NilError(t, path.WriteFile(file.content, 0777))
			}

			titanRoot, packageMode := InferRoot(tt.executionDirectory.RestoreAnchor(fsRoot))
			if !reflect.DeepEqual(titanRoot, tt.rootPath.RestoreAnchor(fsRoot)) {
				t.Errorf("InferRoot() titanRoot = %v, want %v", titanRoot, tt.rootPath.RestoreAnchor(fsRoot))
			}
			if packageMode != tt.packageMode {
				t.Errorf("InferRoot() packageMode = %v, want %v", packageMode, tt.packageMode)
			}
		})
	}
}
