package postgres

import (
	"context"
	"errors"

	"github.com/jackc/pgx/v5"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/gen"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/config"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/rerror"
)

const configLockName = "config"

type Config struct {
	c    *pgxx.Client
	lock repo.Lock
}

var _ repo.Config = (*Config)(nil)

func NewConfig(c *pgxx.Client, lock repo.Lock) *Config {
	return &Config{c: c, lock: lock}
}

func (r *Config) LockAndLoad(ctx context.Context) (*config.Config, error) {
	if err := r.lock.Lock(ctx, configLockName); err != nil {
		return nil, err
	}
	row, err := gen.New(r.c.DB(ctx)).GetConfig(ctx)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return &config.Config{}, nil // mirror Mongo: empty config when absent
		}
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return configFromRow(row), nil
}

func (r *Config) Save(ctx context.Context, cfg *config.Config) error {
	if cfg == nil {
		return nil
	}
	p := gen.UpsertConfigParams{Migration: cfg.Migration}
	if cfg.Auth != nil {
		cert, key := cfg.Auth.Cert, cfg.Auth.Key
		p.AuthCert, p.AuthKey = &cert, &key
	}
	if err := gen.New(r.c.DB(ctx)).UpsertConfig(ctx, p); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func (r *Config) SaveAuth(ctx context.Context, auth *config.Auth) error {
	if auth == nil {
		return nil
	}
	cert, key := auth.Cert, auth.Key
	if err := gen.New(r.c.DB(ctx)).UpsertConfigAuth(ctx, gen.UpsertConfigAuthParams{AuthCert: &cert, AuthKey: &key}); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func (r *Config) SaveAndUnlock(ctx context.Context, cfg *config.Config) error {
	if err := r.Save(ctx, cfg); err != nil {
		return err
	}
	return r.Unlock(ctx)
}

func (r *Config) Unlock(ctx context.Context) error {
	return r.lock.Unlock(ctx, configLockName)
}

func configFromRow(row gen.GetConfigRow) *config.Config {
	cfg := &config.Config{Migration: row.Migration}
	if row.AuthCert != nil || row.AuthKey != nil {
		a := &config.Auth{}
		if row.AuthCert != nil {
			a.Cert = *row.AuthCert
		}
		if row.AuthKey != nil {
			a.Key = *row.AuthKey
		}
		cfg.Auth = a
	}
	return cfg
}
