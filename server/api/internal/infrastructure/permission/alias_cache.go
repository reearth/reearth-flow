package permission

import (
	"context"
	"time"

	expirable "github.com/hashicorp/golang-lru/v2/expirable"
	"golang.org/x/sync/singleflight"
)

const (
	defaultAliasCacheSize = 4096
	defaultAliasCacheTTL  = 5 * time.Minute
)

// aliasCache memoizes workspaceID -> alias lookups process-wide. Aliases
// change rarely and a stale alias fails closed at Cerbos, so staleness is
// not a security concern; boundedness and TTL keep memory in check.
type aliasCache struct {
	lru   *expirable.LRU[string, string]
	group singleflight.Group
}

func newAliasCache(size int, ttl time.Duration) *aliasCache {
	if size <= 0 {
		size = defaultAliasCacheSize
	}
	if ttl <= 0 {
		ttl = defaultAliasCacheTTL
	}
	return &aliasCache{lru: expirable.NewLRU[string, string](size, nil, ttl)}
}

// resolve returns the cached alias for key, calling fetch on a miss.
// Concurrent misses for the same key collapse into a single fetch call.
// Errors from fetch are never cached.
func (c *aliasCache) resolve(ctx context.Context, key string, fetch func(context.Context) (string, error)) (string, error) {
	if alias, ok := c.lru.Get(key); ok {
		return alias, nil
	}

	v, err, _ := c.group.Do(key, func() (interface{}, error) {
		if alias, ok := c.lru.Get(key); ok {
			return alias, nil
		}
		alias, err := fetch(ctx)
		if err != nil {
			return "", err
		}
		c.lru.Add(key, alias)
		return alias, nil
	})
	if err != nil {
		return "", err
	}
	return v.(string), nil
}
