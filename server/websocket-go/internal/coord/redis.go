package coord

import (
	"fmt"
	"log/slog"

	"github.com/reearth/ygo/cluster"

	"github.com/reearth/reearth-flow/websocket-go/internal/config"
	redisrelay "github.com/reearth/reearth-flow/websocket-go/internal/redis"
)

// newRedisRelay builds the Redis Streams relay. Kept in its own file so the
// backend-specific import stays out of coord.go.
func newRedisRelay(cfg *config.Config, flusher Flusher, log *slog.Logger) (cluster.Relay, error) {
	r, err := redisrelay.New(redisrelay.Options{
		URL:     cfg.RedisURL,
		Logger:  log,
		Flusher: flusher,
	})
	if err != nil {
		return nil, fmt.Errorf("build redis relay: %w", err)
	}
	return r, nil
}
