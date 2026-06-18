package postgres

import (
	"context"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/migration"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
	"github.com/reearth/reearthx/authserver"
	"github.com/reearth/reearthx/pgxx"
)

// New builds a repo.Container backed by Postgres. All flow-owned repos are
// implemented; account-owned repos (User/Workspace/Role/Permittable) continue
// to come from the account container.
func New(ctx context.Context, pool *pgxpool.Pool, account *accountrepo.Container) (*repo.Container, error) {
	client := pgxx.NewClient(pool, pgxx.WithTxRetry(2)) // retry serialization failures, matching the Mongo path
	lock := NewLock(pool)
	c := &repo.Container{
		Trigger:       NewTrigger(client),
		Config:        NewConfig(client, lock),
		Parameter:     NewParameter(client),
		WorkerConfig:  NewWorkerConfig(client),
		ProjectAccess: NewProjectAccess(client),
		Workflow:      NewWorkflow(client),
		Project:       NewProject(client),
		Deployment:    NewDeployment(client),
		EdgeExecution: NewEdgeExecution(client),
		NodeExecution: NewNodeExecution(client),
		Job:           NewJob(client),
		Asset:         NewAsset(client),
		AssetUpload:   NewAssetUpload(client),
		AuthRequest:   authserver.NewPostgres(client),
		Lock:          lock,
		Transaction:   client,
		Workspace:     account.Workspace,
		User:          account.User,
		Role:          account.Role,
		Permittable:   account.Permittable,
	}
	if err := migration.Do(ctx, client, c.Config); err != nil {
		return nil, err
	}
	return c, nil
}
