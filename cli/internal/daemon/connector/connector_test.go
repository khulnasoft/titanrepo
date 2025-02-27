package connector

import (
	"context"
	"errors"
	"net"
	"os/exec"
	"runtime"
	"strconv"
	"testing"

	"github.com/hashicorp/go-hclog"
	"github.com/khulnasoft/titanrepo/cli/internal/fs"
	"github.com/khulnasoft/titanrepo/cli/internal/titandprotocol"
	"github.com/khulnasoft/titanrepo/cli/internal/titanpath"
	"github.com/nightlyone/lockfile"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/test/bufconn"
	"gotest.tools/v3/assert"
)

// testBin returns a platform-appropriate executable to run node.
// Node works here as an arbitrary process to start, since it's
// required for titan development. It will obviously not implement
// our grpc service, use a mockServer instance where that's needed.
func testBin() string {
	if runtime.GOOS == "windows" {
		return "node.exe"
	}
	return "node"
}

func getUnixSocket(dir titanpath.AbsoluteSystemPath) titanpath.AbsoluteSystemPath {
	return dir.UntypedJoin("titand-test.sock")
}

func getPidFile(dir titanpath.AbsoluteSystemPath) titanpath.AbsoluteSystemPath {
	return dir.UntypedJoin("titand-test.pid")
}

func TestConnectFailsWithoutGrpcServer(t *testing.T) {
	// We aren't starting a server that is going to write
	// to our socket file, so we should see a series of connection
	// failures, followed by ErrTooManyAttempts
	logger := hclog.Default()
	dir := t.TempDir()
	dirPath := fs.AbsoluteSystemPathFromUpstream(dir)

	sockPath := getUnixSocket(dirPath)
	pidPath := getPidFile(dirPath)
	ctx := context.Background()
	bin := testBin()
	c := &Connector{
		Logger:   logger,
		Bin:      bin,
		Opts:     Opts{},
		SockPath: sockPath,
		PidPath:  pidPath,
	}
	// Note that we expect ~3s here, for 3 attempts with a timeout of 1s
	_, err := c.connectInternal(ctx)
	assert.ErrorIs(t, err, ErrTooManyAttempts)
}

func TestKillDeadServerNoPid(t *testing.T) {
	logger := hclog.Default()
	dir := t.TempDir()
	dirPath := fs.AbsoluteSystemPathFromUpstream(dir)

	sockPath := getUnixSocket(dirPath)
	pidPath := getPidFile(dirPath)
	c := &Connector{
		Logger:   logger,
		Bin:      "nonexistent",
		Opts:     Opts{},
		SockPath: sockPath,
		PidPath:  pidPath,
	}

	err := c.killDeadServer(99999)
	assert.NilError(t, err, "killDeadServer")
}

func TestKillDeadServerNoProcess(t *testing.T) {
	logger := hclog.Default()
	dir := t.TempDir()
	dirPath := fs.AbsoluteSystemPathFromUpstream(dir)

	sockPath := getUnixSocket(dirPath)
	pidPath := getPidFile(dirPath)
	// Simulate the socket already existing, with no live daemon
	err := sockPath.WriteFile([]byte("junk"), 0644)
	assert.NilError(t, err, "WriteFile")
	err = pidPath.WriteFile([]byte("99999"), 0644)
	assert.NilError(t, err, "WriteFile")
	c := &Connector{
		Logger:   logger,
		Bin:      "nonexistent",
		Opts:     Opts{},
		SockPath: sockPath,
		PidPath:  pidPath,
	}

	err = c.killDeadServer(99999)
	assert.ErrorIs(t, err, lockfile.ErrDeadOwner)
	stillExists := pidPath.FileExists()
	if !stillExists {
		t.Error("pidPath should still exist, expected the user to clean it up")
	}
}

func TestKillDeadServerWithProcess(t *testing.T) {
	logger := hclog.Default()
	dir := t.TempDir()
	dirPath := fs.AbsoluteSystemPathFromUpstream(dir)

	sockPath := getUnixSocket(dirPath)
	pidPath := getPidFile(dirPath)
	// Simulate the socket already existing, with no live daemon
	err := sockPath.WriteFile([]byte("junk"), 0644)
	assert.NilError(t, err, "WriteFile")
	bin := testBin()
	cmd := exec.Command(bin)
	err = cmd.Start()
	assert.NilError(t, err, "cmd.Start")
	pid := cmd.Process.Pid
	if pid == 0 {
		t.Fatalf("failed to start process %v", bin)
	}

	err = pidPath.WriteFile([]byte(strconv.Itoa(pid)), 0644)
	assert.NilError(t, err, "WriteFile")
	c := &Connector{
		Logger:   logger,
		Bin:      "nonexistent",
		Opts:     Opts{},
		SockPath: sockPath,
		PidPath:  pidPath,
	}

	err = c.killDeadServer(pid)
	assert.NilError(t, err, "killDeadServer")
	stillExists := pidPath.FileExists()
	if !stillExists {
		t.Error("pidPath no longer exists, expected client to not clean it up")
	}
	err = cmd.Wait()
	exitErr := &exec.ExitError{}
	if !errors.As(err, &exitErr) {
		t.Errorf("expected an exit error from %v, got %v", bin, err)
	}
}

type mockServer struct {
	titandprotocol.UnimplementedTurbodServer
	helloErr     error
	shutdownResp *titandprotocol.ShutdownResponse
	pidFile      titanpath.AbsoluteSystemPath
}

// Simulates server exiting by cleaning up the pid file
func (s *mockServer) Shutdown(ctx context.Context, req *titandprotocol.ShutdownRequest) (*titandprotocol.ShutdownResponse, error) {
	if err := s.pidFile.Remove(); err != nil {
		return nil, err
	}
	return s.shutdownResp, nil
}

func (s *mockServer) Hello(ctx context.Context, req *titandprotocol.HelloRequest) (*titandprotocol.HelloResponse, error) {
	if req.Version == "" {
		return nil, errors.New("missing version")
	}
	return nil, s.helloErr
}

func TestKillLiveServer(t *testing.T) {
	logger := hclog.Default()
	dir := t.TempDir()
	dirPath := fs.AbsoluteSystemPathFromUpstream(dir)

	sockPath := getUnixSocket(dirPath)
	pidPath := getPidFile(dirPath)
	err := pidPath.WriteFile([]byte("99999"), 0644)
	assert.NilError(t, err, "WriteFile")

	ctx := context.Background()
	c := &Connector{
		Logger:       logger,
		Bin:          "nonexistent",
		Opts:         Opts{},
		SockPath:     sockPath,
		PidPath:      pidPath,
		TurboVersion: "some-version",
	}

	st := status.New(codes.FailedPrecondition, "version mismatch")
	mock := &mockServer{
		shutdownResp: &titandprotocol.ShutdownResponse{},
		helloErr:     st.Err(),
		pidFile:      pidPath,
	}
	lis := bufconn.Listen(1024 * 1024)
	grpcServer := grpc.NewServer()
	titandprotocol.RegisterTurbodServer(grpcServer, mock)
	go func(t *testing.T) {
		if err := grpcServer.Serve(lis); err != nil {
			t.Logf("server closed: %v", err)
		}
	}(t)

	conn, err := grpc.DialContext(ctx, "bufnet", grpc.WithContextDialer(func(ctx context.Context, s string) (net.Conn, error) {
		return lis.Dial()
	}), grpc.WithTransportCredentials(insecure.NewCredentials()))
	assert.NilError(t, err, "DialContext")
	titanClient := titandprotocol.NewTurbodClient(conn)
	client := &Client{
		TurbodClient: titanClient,
		ClientConn:   conn,
	}
	err = c.sendHello(ctx, client)
	if !errors.Is(err, errVersionMismatch) {
		t.Errorf("sendHello error got %v, want %v", err, errVersionMismatch)
	}
	err = c.killLiveServer(ctx, client, 99999)
	assert.NilError(t, err, "killLiveServer")
	// Expect the pid file and socket files to have been cleaned up
	if pidPath.FileExists() {
		t.Errorf("expected pid file to have been deleted: %v", pidPath)
	}
	if sockPath.FileExists() {
		t.Errorf("expected socket file to have been deleted: %v", sockPath)
	}
}
