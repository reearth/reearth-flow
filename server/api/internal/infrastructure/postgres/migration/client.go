package migration

import (
	"context"
	"errors"
	"sync"

	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/config"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/usecasex/migration"
)

// DBClient is what migration funcs receive — the pgx Client, used to run SQL via
// DBClient.DB(ctx). It is also the usecasex.Transactor that drives the Runner.
type DBClient = *pgxx.Client

// Do runs pending Postgres data/code migrations using the Transactor-based
// migration.Runner (Postgres path), mirroring the Mongo migration.Do.
func Do(ctx context.Context, client *pgxx.Client, config repo.Config) error {
	return migration.NewRunner[DBClient](client, client, NewConfig(config), migrations).Migrate(ctx)
}

// Config adapts repo.Config to migration.ConfigRepo (version tracking via the
// config row's Migration field, guarded by the config lock). Mirrors the Mongo
// adapter.
type Config struct {
	c       repo.Config
	current config.Config
	m       sync.Mutex
	locked  bool
}

func NewConfig(c repo.Config) *Config {
	return &Config{c: c}
}

func (c *Config) Begin(ctx context.Context) error {
	c.m.Lock()
	defer c.m.Unlock()

	conf, err := c.c.LockAndLoad(ctx)
	if conf != nil {
		c.current = *conf
	}
	if err == nil {
		c.locked = true
	}
	return err
}

func (c *Config) End(ctx context.Context) error {
	c.m.Lock()
	defer c.m.Unlock()

	if err := c.c.Unlock(ctx); err != nil {
		return err
	}
	c.locked = false
	return nil
}

func (c *Config) Current(ctx context.Context) (migration.Key, error) {
	if !c.locked {
		return 0, errors.New("config is not locked")
	}
	return c.current.Migration, nil
}

func (c *Config) Save(ctx context.Context, key migration.Key) error {
	c.m.Lock()
	defer c.m.Unlock()

	c.current.Migration = key
	return c.c.Save(ctx, &c.current)
}
