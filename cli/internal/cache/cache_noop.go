package cache

import "github.com/khulnasoft/titanrepo/cli/internal/titanpath"

type noopCache struct{}

func newNoopCache() *noopCache {
	return &noopCache{}
}

func (c *noopCache) Put(anchor titanpath.AbsoluteSystemPath, key string, duration int, files []titanpath.AnchoredSystemPath) error {
	return nil
}
func (c *noopCache) Fetch(anchor titanpath.AbsoluteSystemPath, key string, files []string) (bool, []titanpath.AnchoredSystemPath, int, error) {
	return false, nil, 0, nil
}
func (c *noopCache) Exists(key string) (ItemStatus, error) {
	return ItemStatus{}, nil
}

func (c *noopCache) Clean(anchor titanpath.AbsoluteSystemPath) {}
func (c *noopCache) CleanAll()                                 {}
func (c *noopCache) Shutdown()                                 {}
