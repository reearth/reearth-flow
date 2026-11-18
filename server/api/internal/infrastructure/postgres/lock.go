package postgres

import (
	"context"
	"hash/fnv"
	"sync"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearthx/pgxx"
)

// Lock implements repo.Lock with Postgres session-level advisory locks, each
// holding a pooled connection until Unlock. It suits coarse, short-lived
// coordination such as the config lock.
type Lock struct {
	pool  *pgxpool.Pool
	locks sync.Map // name -> pgxx.Unlock
}

var _ repo.Lock = (*Lock)(nil)

func NewLock(pool *pgxpool.Pool) *Lock {
	return &Lock{pool: pool}
}

func (r *Lock) Lock(ctx context.Context, name string) error {
	if _, held := r.locks.Load(name); held {
		return repo.ErrAlreadyLocked
	}
	unlock, acquired, err := pgxx.TryAdvisoryLock(ctx, r.pool, lockKey(name))
	if err != nil || !acquired {
		return repo.ErrFailedToLock
	}
	r.locks.Store(name, unlock)
	return nil
}

func (r *Lock) Unlock(ctx context.Context, name string) error {
	v, held := r.locks.LoadAndDelete(name)
	if !held {
		return repo.ErrNotLocked
	}
	return v.(pgxx.Unlock)(ctx)
}

func lockKey(name string) int64 {
	h := fnv.New64a()
	_, _ = h.Write([]byte(name))
	return int64(h.Sum64())
}
