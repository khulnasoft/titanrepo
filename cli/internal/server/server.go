package server

import (
	"context"
	"sync"
	"time"

	"github.com/hashicorp/go-hclog"
	"github.com/khulnasoft/titanrepo/cli/internal/filewatcher"
	"github.com/khulnasoft/titanrepo/cli/internal/fs"
	"github.com/khulnasoft/titanrepo/cli/internal/globwatcher"
	"github.com/khulnasoft/titanrepo/cli/internal/titandprotocol"
	"github.com/khulnasoft/titanrepo/cli/internal/titanpath"
	"github.com/pkg/errors"
	"google.golang.org/grpc"
	codes "google.golang.org/grpc/codes"
	status "google.golang.org/grpc/status"
)

// Server implements the GRPC serverside of TitandServer
// Note for the future: we don't yet make use of titan.json
// or the package graph in the server. Once we do, we may need a
// layer of indirection between "the thing that responds to grpc requests"
// and "the thing that holds our persistent data structures" to handle
// changes in the underlying configuration.
type Server struct {
	titandprotocol.UnimplementedTitandServer
	watcher      *filewatcher.FileWatcher
	globWatcher  *globwatcher.GlobWatcher
	titanVersion string
	started      time.Time
	logFilePath  titanpath.AbsoluteSystemPath
	repoRoot     titanpath.AbsoluteSystemPath
	closerMu     sync.Mutex
	closer       *closer
}

// GRPCServer is the interface that the titan server needs to the underlying
// GRPC server. This lets the titan server register itself, as well as provides
// a hook for shutting down the server.
type GRPCServer interface {
	grpc.ServiceRegistrar
	GracefulStop()
}

type closer struct {
	grpcServer GRPCServer
	once       sync.Once
}

func (c *closer) close() {
	// This can get triggered from a request handler (Shutdown). Since
	// calling GracefulStop blocks until all request handlers complete,
	// we need to run it in a goroutine to let the Shutdown handler complete
	// and avoid deadlocking.
	c.once.Do(func() {
		go func() {
			c.grpcServer.GracefulStop()
		}()
	})
}

var _defaultCookieTimeout = 500 * time.Millisecond

// New returns a new instance of Server
func New(serverName string, logger hclog.Logger, repoRoot titanpath.AbsoluteSystemPath, titanVersion string, logFilePath titanpath.AbsoluteSystemPath) (*Server, error) {
	cookieDir := fs.GetTitanDataDir().UntypedJoin("cookies", serverName)
	cookieJar, err := filewatcher.NewCookieJar(cookieDir, _defaultCookieTimeout)
	if err != nil {
		return nil, err
	}
	watcher, err := filewatcher.GetPlatformSpecificBackend(logger)
	if err != nil {
		return nil, err
	}
	fileWatcher := filewatcher.New(logger.Named("FileWatcher"), repoRoot, watcher)
	globWatcher := globwatcher.New(logger.Named("GlobWatcher"), repoRoot, cookieJar)
	server := &Server{
		watcher:      fileWatcher,
		globWatcher:  globWatcher,
		titanVersion: titanVersion,
		started:      time.Now(),
		logFilePath:  logFilePath,
		repoRoot:     repoRoot,
	}
	server.watcher.AddClient(cookieJar)
	server.watcher.AddClient(globWatcher)
	server.watcher.AddClient(server)
	if err := server.watcher.Start(); err != nil {
		return nil, errors.Wrapf(err, "watching %v", repoRoot)
	}
	if err := server.watcher.AddRoot(cookieDir); err != nil {
		_ = server.watcher.Close()
		return nil, errors.Wrapf(err, "failed to watch cookie directory: %v", cookieDir)
	}
	return server, nil
}

func (s *Server) tryClose() bool {
	s.closerMu.Lock()
	defer s.closerMu.Unlock()
	if s.closer != nil {
		s.closer.close()
		return true
	}
	return false
}

// OnFileWatchEvent implements filewatcher.FileWatchClient.OnFileWatchEvent
// In the event that the root of the monorepo is deleted, shut down the server.
func (s *Server) OnFileWatchEvent(ev filewatcher.Event) {
	if ev.EventType == filewatcher.FileDeleted && ev.Path == s.repoRoot {
		_ = s.tryClose()
	}
}

// OnFileWatchError implements filewatcher.FileWatchClient.OnFileWatchError
func (s *Server) OnFileWatchError(err error) {}

// OnFileWatchClosed implements filewatcher.FileWatchClient.OnFileWatchClosed
func (s *Server) OnFileWatchClosed() {}

// Close is used for shutting down this copy of the server
func (s *Server) Close() error {
	return s.watcher.Close()
}

// Register registers this server to respond to GRPC requests
func (s *Server) Register(grpcServer GRPCServer) {
	s.closerMu.Lock()
	s.closer = &closer{
		grpcServer: grpcServer,
	}
	s.closerMu.Unlock()
	titandprotocol.RegisterTitandServer(grpcServer, s)
}

// NotifyOutputsWritten implements the NotifyOutputsWritten rpc from titan.proto
func (s *Server) NotifyOutputsWritten(ctx context.Context, req *titandprotocol.NotifyOutputsWrittenRequest) (*titandprotocol.NotifyOutputsWrittenResponse, error) {
	outputs := fs.TaskOutputs{
		Inclusions: req.OutputGlobs,
		Exclusions: req.OutputExclusionGlobs,
	}

	err := s.globWatcher.WatchGlobs(req.Hash, outputs)
	if err != nil {
		return nil, err
	}
	return &titandprotocol.NotifyOutputsWrittenResponse{}, nil
}

// GetChangedOutputs implements the GetChangedOutputs rpc from titan.proto
func (s *Server) GetChangedOutputs(ctx context.Context, req *titandprotocol.GetChangedOutputsRequest) (*titandprotocol.GetChangedOutputsResponse, error) {

	changedGlobs, err := s.globWatcher.GetChangedGlobs(req.Hash, req.OutputGlobs)
	if err != nil {
		return nil, err
	}
	return &titandprotocol.GetChangedOutputsResponse{
		ChangedOutputGlobs: changedGlobs,
	}, nil
}

// Hello implements the Hello rpc from titan.proto
func (s *Server) Hello(ctx context.Context, req *titandprotocol.HelloRequest) (*titandprotocol.HelloResponse, error) {
	clientVersion := req.Version
	if clientVersion != s.titanVersion {
		err := status.Errorf(codes.FailedPrecondition, "version mismatch. Client %v Server %v", clientVersion, s.titanVersion)
		return nil, err
	}
	return &titandprotocol.HelloResponse{}, nil
}

// Shutdown implements the Shutdown rpc from titan.proto
func (s *Server) Shutdown(ctx context.Context, req *titandprotocol.ShutdownRequest) (*titandprotocol.ShutdownResponse, error) {
	if s.tryClose() {
		return &titandprotocol.ShutdownResponse{}, nil
	}
	err := status.Error(codes.NotFound, "shutdown mechanism not found")
	return nil, err
}

// Status implements the Status rpc from titan.proto
func (s *Server) Status(ctx context.Context, req *titandprotocol.StatusRequest) (*titandprotocol.StatusResponse, error) {
	uptime := uint64(time.Since(s.started).Milliseconds())
	return &titandprotocol.StatusResponse{
		DaemonStatus: &titandprotocol.DaemonStatus{
			LogFile:    s.logFilePath.ToString(),
			UptimeMsec: uptime,
		},
	}, nil
}
