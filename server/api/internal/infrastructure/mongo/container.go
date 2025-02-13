package mongo

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/migration"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountinfrastructure/accountmongo"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
	"github.com/reearth/reearthx/authserver"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/util"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
)

func New(ctx context.Context, db *mongo.Database, account *accountrepo.Container, useTransaction bool) (*repo.Container, error) {
	lock, err := NewLock(db.Collection("locks"))
	if err != nil {
		return nil, err
	}

	client := mongox.NewClientWithDatabase(db)
	if useTransaction {
		client = client.WithTransaction()
	}

	c := &repo.Container{
		Asset:         NewAsset(client),
		AuthRequest:   authserver.NewMongo(client.WithCollection("authRequest")),
		Config:        NewConfig(db.Collection("config"), lock),
		Deployment:    NewDeployment(client),
		Job:           NewJob(client),
		Workflow:      NewWorkflow(client),
		Parameter:     NewParameter(client),
		Project:       NewProject(client),
		ProjectAccess: NewProjectAccess(client),
		Lock:          lock,
		Transaction:   client.Transaction(),
		Trigger:       NewTrigger(client),
		Workspace:     account.Workspace,
		User:          account.User,
	}

	if err := Init(c); err != nil {
		return nil, err
	}

	if err := migration.Do(ctx, client, c.Config); err != nil {
		return nil, err
	}

	return c, nil
}

func Init(r *repo.Container) error {
	if r == nil {
		return nil
	}

	ctx := context.Background()
	return util.Try(
		func() error { return r.Asset.(*Asset).Init(ctx) },
		func() error { return r.AuthRequest.(*authserver.Mongo).Init(ctx) },
		func() error { return r.Workflow.(*Workflow).Init(ctx) },
		func() error { return r.Deployment.(*DeploymentAdapter).Deployment.Init(ctx) },
		func() error { return r.Job.(*Job).Init(ctx) },
		func() error { return r.Parameter.(*Parameter).Init(ctx) },
		func() error { return r.Project.(*Project).Init(ctx) },
		func() error { return r.ProjectAccess.(*ProjectAccess).Init(ctx) },
		func() error { return r.Trigger.(*Trigger).Init(ctx) },
		func() error { return r.User.(*accountmongo.User).Init() },
		func() error { return r.Workspace.(*accountmongo.Workspace).Init() },
	)
}

func applyWorkspaceFilter(filter interface{}, ids user.WorkspaceIDList) interface{} {
	if ids == nil {
		return filter
	}
	return mongox.And(filter, "workspace", bson.M{"$in": ids.Strings()})
}

func createIndexes(ctx context.Context, c *mongox.ClientCollection, keys, uniqueKeys []string) error {
	created, deleted, err := c.Indexes(ctx, keys, uniqueKeys)
	if len(created) > 0 || len(deleted) > 0 {
		log.Infofc(ctx, "mongo: %s: index deleted: %v, created: %v\n", c.Client().Name(), deleted, created)
	}
	return err
}
