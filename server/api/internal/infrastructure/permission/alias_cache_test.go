package permission

import (
	"context"
	"fmt"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestAliasCache_SecondLookupSameKey_MakesNoAdditionalCall(t *testing.T) {
	c := newAliasCache(defaultAliasCacheSize, time.Minute)
	var calls int32

	fetch := func(_ context.Context) (string, error) {
		atomic.AddInt32(&calls, 1)
		return "acme", nil
	}

	alias, err := c.resolve(context.Background(), "ws-1", fetch)
	require.NoError(t, err)
	assert.Equal(t, "acme", alias)

	alias, err = c.resolve(context.Background(), "ws-1", fetch)
	require.NoError(t, err)
	assert.Equal(t, "acme", alias)

	assert.Equal(t, int32(1), atomic.LoadInt32(&calls), "second lookup for same key must not hit fetch again")
}

func TestAliasCache_DifferentKey_MakesOneCallEach(t *testing.T) {
	c := newAliasCache(defaultAliasCacheSize, time.Minute)
	var calls int32

	fetch := func(key string) func(context.Context) (string, error) {
		return func(_ context.Context) (string, error) {
			atomic.AddInt32(&calls, 1)
			return key + "-alias", nil
		}
	}

	alias1, err := c.resolve(context.Background(), "ws-1", fetch("ws-1"))
	require.NoError(t, err)
	assert.Equal(t, "ws-1-alias", alias1)

	alias2, err := c.resolve(context.Background(), "ws-2", fetch("ws-2"))
	require.NoError(t, err)
	assert.Equal(t, "ws-2-alias", alias2)

	assert.Equal(t, int32(2), atomic.LoadInt32(&calls), "a different key must trigger its own fetch")
}

func TestAliasCache_EntriesExpireAfterTTL(t *testing.T) {
	c := newAliasCache(defaultAliasCacheSize, 20*time.Millisecond)
	var calls int32

	fetch := func(_ context.Context) (string, error) {
		atomic.AddInt32(&calls, 1)
		return "acme", nil
	}

	_, err := c.resolve(context.Background(), "ws-1", fetch)
	require.NoError(t, err)

	time.Sleep(200 * time.Millisecond)

	_, err = c.resolve(context.Background(), "ws-1", fetch)
	require.NoError(t, err)

	assert.Equal(t, int32(2), atomic.LoadInt32(&calls), "entry must be refetched after the TTL elapses")
}

func TestAliasCache_ConcurrentMisses_CollapseIntoOneUpstreamCall(t *testing.T) {
	c := newAliasCache(defaultAliasCacheSize, time.Minute)
	var calls int32
	release := make(chan struct{})
	started := make(chan struct{})
	var startOnce sync.Once

	fetch := func(_ context.Context) (string, error) {
		atomic.AddInt32(&calls, 1)
		startOnce.Do(func() { close(started) })
		<-release
		return "acme", nil
	}

	const n = 20
	var wg sync.WaitGroup
	results := make([]string, n)
	errs := make([]error, n)
	for i := 0; i < n; i++ {
		wg.Add(1)
		go func(i int) {
			defer wg.Done()
			alias, err := c.resolve(context.Background(), "ws-1", fetch)
			results[i] = alias
			errs[i] = err
		}(i)
	}

	<-started
	close(release)
	wg.Wait()

	for i := 0; i < n; i++ {
		require.NoError(t, errs[i])
		assert.Equal(t, "acme", results[i])
	}
	assert.Equal(t, int32(1), atomic.LoadInt32(&calls), "concurrent misses for the same key must collapse into a single upstream call")
}

func TestAliasCache_ErrorsAreNotCached(t *testing.T) {
	c := newAliasCache(defaultAliasCacheSize, time.Minute)
	var calls int32

	fetch := func(_ context.Context) (string, error) {
		n := atomic.AddInt32(&calls, 1)
		if n == 1 {
			return "", fmt.Errorf("boom")
		}
		return "acme", nil
	}

	_, err := c.resolve(context.Background(), "ws-1", fetch)
	require.Error(t, err)

	alias, err := c.resolve(context.Background(), "ws-1", fetch)
	require.NoError(t, err)
	assert.Equal(t, "acme", alias)
	assert.Equal(t, int32(2), atomic.LoadInt32(&calls), "a failed fetch must not be cached")
}

func TestAliasCache_BoundedSize_EvictsOldestOnOverflow(t *testing.T) {
	const capacity = 4
	c := newAliasCache(capacity, time.Minute)

	fetch := func(key string) func(context.Context) (string, error) {
		return func(_ context.Context) (string, error) {
			return key + "-alias", nil
		}
	}

	for i := 0; i < capacity+2; i++ {
		key := fmt.Sprintf("ws-%d", i)
		_, err := c.resolve(context.Background(), key, fetch(key))
		require.NoError(t, err)
	}

	assert.LessOrEqual(t, c.lru.Len(), capacity, "cache must stay within its configured capacity")

	var calls int32
	_, err := c.resolve(context.Background(), "ws-0", func(_ context.Context) (string, error) {
		atomic.AddInt32(&calls, 1)
		return "ws-0-alias", nil
	})
	require.NoError(t, err)
	assert.Equal(t, int32(1), atomic.LoadInt32(&calls), "the oldest entry must have been evicted, forcing a refetch")
}
