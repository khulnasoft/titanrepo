// Package daemonclient is a wrapper around a grpc client
// to talk to titand
package daemonclient

import (
	"context"

	"github.com/khulnasoft/titanrepo/cli/internal/daemon/connector"
	"github.com/khulnasoft/titanrepo/cli/internal/fs"
	"github.com/khulnasoft/titanrepo/cli/internal/titandprotocol"
	"github.com/khulnasoft/titanrepo/cli/internal/titanpath"
)

// DaemonClient provides access to higher-level functionality from the daemon to a titan run.
type DaemonClient struct {
	client *connector.Client
}

// Status provides details about the daemon's status
type Status struct {
	UptimeMs uint64                       `json:"uptimeMs"`
	LogFile  titanpath.AbsoluteSystemPath `json:"logFile"`
	PidFile  titanpath.AbsoluteSystemPath `json:"pidFile"`
	SockFile titanpath.AbsoluteSystemPath `json:"sockFile"`
}

// New creates a new instance of a DaemonClient.
func New(client *connector.Client) *DaemonClient {
	return &DaemonClient{
		client: client,
	}
}

// GetChangedOutputs implements runcache.OutputWatcher.GetChangedOutputs
func (d *DaemonClient) GetChangedOutputs(ctx context.Context, hash string, repoRelativeOutputGlobs []string) ([]string, error) {
	resp, err := d.client.GetChangedOutputs(ctx, &titandprotocol.GetChangedOutputsRequest{
		Hash:        hash,
		OutputGlobs: repoRelativeOutputGlobs,
	})
	if err != nil {
		return nil, err
	}

	return resp.ChangedOutputGlobs, nil
}

// NotifyOutputsWritten implements runcache.OutputWatcher.NotifyOutputsWritten
func (d *DaemonClient) NotifyOutputsWritten(ctx context.Context, hash string, repoRelativeOutputGlobs fs.TaskOutputs) error {
	_, err := d.client.NotifyOutputsWritten(ctx, &titandprotocol.NotifyOutputsWrittenRequest{
		Hash:                 hash,
		OutputGlobs:          repoRelativeOutputGlobs.Inclusions,
		OutputExclusionGlobs: repoRelativeOutputGlobs.Exclusions,
	})
	return err
}

// Status returns the DaemonStatus from the daemon
func (d *DaemonClient) Status(ctx context.Context) (*Status, error) {
	resp, err := d.client.Status(ctx, &titandprotocol.StatusRequest{})
	if err != nil {
		return nil, err
	}
	daemonStatus := resp.DaemonStatus
	return &Status{
		UptimeMs: daemonStatus.UptimeMsec,
		LogFile:  d.client.LogPath,
		PidFile:  d.client.PidPath,
		SockFile: d.client.SockPath,
	}, nil
}
