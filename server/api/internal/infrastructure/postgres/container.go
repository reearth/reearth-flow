package postgres

import (
	"context"
	"fmt"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/migration"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
	"github.com/reearth/reearthx/pgxx"
)

// New builds a repo.Container backed by Postgres. During the incremental
// migration (design A1) only ported entities are implemented here; the
// remaining repos are left nil and mustComplete fails fast so that
// DB_DRIVER=postgres cannot boot into production until every entity is ported.
// Account-owned repos (User/Workspace/Role/Permittable) continue to come from
// the account container until they are ported in a separate stream.
func New(ctx context.Context, pool *pgxpool.Pool, account *accountrepo.Container) (*repo.Container, error) {
	client := pgxx.NewClient(pool, pgxx.WithTxRetry(2)) // retry serialization failures, matching the Mongo path
	lock := NewLock(pool)
	c := &repo.Container{
		Trigger:       NewTrigger(client),
		Config:        NewConfig(client, lock),
		Parameter:     NewParameter(client),
		WorkerConfig:  NewWorkerConfig(client),
		ProjectAccess: NewProjectAccess(client),
		Lock:          lock,
		Transaction:   client,
		Workspace:     account.Workspace,
		User:          account.User,
		Role:          account.Role,
		Permittable:   account.Permittable,
	}
	if err := mustComplete(c); err != nil {
		return nil, err
	}
	if err := migration.Do(ctx, client, c.Config); err != nil {
		return nil, err
	}
	return c, nil
}

// mustComplete reports which flow-owned repos are not yet implemented for
// Postgres. Remove an entry from this list as each repo is ported; delete the
// whole guard at the final cutover.
func mustComplete(c *repo.Container) error {
	missing := []string{}
	check := func(name string, ok bool) {
		if !ok {
			missing = append(missing, name)
		}
	}
	check("Asset", c.Asset != nil)
	check("AssetUpload", c.AssetUpload != nil)
	check("AuthRequest", c.AuthRequest != nil)
	check("Deployment", c.Deployment != nil)
	check("EdgeExecution", c.EdgeExecution != nil)
	check("Job", c.Job != nil)
	check("NodeExecution", c.NodeExecution != nil)
	check("Project", c.Project != nil)
	check("Workflow", c.Workflow != nil)
	if len(missing) > 0 {
		return fmt.Errorf("postgres backend not yet implemented for: %v (set DB_DRIVER=mongo)", missing)
	}
	return nil
}
