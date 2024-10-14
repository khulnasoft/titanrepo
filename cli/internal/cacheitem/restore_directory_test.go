package cacheitem

import (
	"reflect"
	"testing"

	"github.com/khulnasoft/titanrepo/cli/internal/titanpath"
)

func Test_cachedDirTree_getStartingPoint(t *testing.T) {
	testDir := titanpath.AbsoluteSystemPath("")
	tests := []struct {
		name string

		// STATE
		cachedDirTree cachedDirTree

		// INPUT
		path titanpath.AnchoredSystemPath

		// OUTPUT
		calculatedAnchor titanpath.AbsoluteSystemPath
		pathSegments     []titanpath.RelativeSystemPath
	}{
		{
			name: "hello world",
			cachedDirTree: cachedDirTree{
				anchorAtDepth: []titanpath.AbsoluteSystemPath{testDir},
				prefix:        []titanpath.RelativeSystemPath{},
			},
			path:             titanpath.AnchoredUnixPath("hello/world").ToSystemPath(),
			calculatedAnchor: testDir,
			pathSegments:     []titanpath.RelativeSystemPath{"hello", "world"},
		},
		{
			name: "has a cache",
			cachedDirTree: cachedDirTree{
				anchorAtDepth: []titanpath.AbsoluteSystemPath{
					testDir,
					testDir.UntypedJoin("hello"),
				},
				prefix: []titanpath.RelativeSystemPath{"hello"},
			},
			path:             titanpath.AnchoredUnixPath("hello/world").ToSystemPath(),
			calculatedAnchor: testDir.UntypedJoin("hello"),
			pathSegments:     []titanpath.RelativeSystemPath{"world"},
		},
		{
			name: "ask for yourself",
			cachedDirTree: cachedDirTree{
				anchorAtDepth: []titanpath.AbsoluteSystemPath{
					testDir,
					testDir.UntypedJoin("hello"),
					testDir.UntypedJoin("hello", "world"),
				},
				prefix: []titanpath.RelativeSystemPath{"hello", "world"},
			},
			path:             titanpath.AnchoredUnixPath("hello/world").ToSystemPath(),
			calculatedAnchor: testDir.UntypedJoin("hello", "world"),
			pathSegments:     []titanpath.RelativeSystemPath{},
		},
		{
			name: "three layer cake",
			cachedDirTree: cachedDirTree{
				anchorAtDepth: []titanpath.AbsoluteSystemPath{
					testDir,
					testDir.UntypedJoin("hello"),
					testDir.UntypedJoin("hello", "world"),
				},
				prefix: []titanpath.RelativeSystemPath{"hello", "world"},
			},
			path:             titanpath.AnchoredUnixPath("hello/world/again").ToSystemPath(),
			calculatedAnchor: testDir.UntypedJoin("hello", "world"),
			pathSegments:     []titanpath.RelativeSystemPath{"again"},
		},
		{
			name: "outside of cache hierarchy",
			cachedDirTree: cachedDirTree{
				anchorAtDepth: []titanpath.AbsoluteSystemPath{
					testDir,
					testDir.UntypedJoin("hello"),
					testDir.UntypedJoin("hello", "world"),
				},
				prefix: []titanpath.RelativeSystemPath{"hello", "world"},
			},
			path:             titanpath.AnchoredUnixPath("somewhere/else").ToSystemPath(),
			calculatedAnchor: testDir,
			pathSegments:     []titanpath.RelativeSystemPath{"somewhere", "else"},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			cr := tt.cachedDirTree
			calculatedAnchor, pathSegments := cr.getStartingPoint(tt.path)
			if !reflect.DeepEqual(calculatedAnchor, tt.calculatedAnchor) {
				t.Errorf("cachedDirTree.getStartingPoint() calculatedAnchor = %v, want %v", calculatedAnchor, tt.calculatedAnchor)
			}
			if !reflect.DeepEqual(pathSegments, tt.pathSegments) {
				t.Errorf("cachedDirTree.getStartingPoint() pathSegments = %v, want %v", pathSegments, tt.pathSegments)
			}
		})
	}
}
