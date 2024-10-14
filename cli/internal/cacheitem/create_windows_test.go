//go:build windows
// +build windows

package cacheitem

import (
	"testing"

	"github.com/khulnasoft/titanrepo/cli/internal/titanpath"
)

func createFifo(t *testing.T, anchor titanpath.AbsoluteSystemPath, fileDefinition createFileDefinition) error {
	return errUnsupportedFileType
}
