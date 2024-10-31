package mongo

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
	"go.mongodb.org/mongo-driver/bson"
)

var (
	deploymentIndexes       = []string{"workspaceid"}
	deploymentUniqueIndexes = []string{"id"}
)

type Deployment struct {
	client *mongox.ClientCollection
	f      repo.WorkspaceFilter
}

func NewDeployment(client *mongox.Client) *Deployment {
	return &Deployment{client: client.WithCollection("deployment")}
}

func (r *Deployment) Init(ctx context.Context) error {
	return createIndexes(ctx, r.client, deploymentIndexes, deploymentUniqueIndexes)
}

func (r *Deployment) Filtered(f repo.WorkspaceFilter) repo.Deployment {
	return &Deployment{
		client: r.client,
		f:      r.f.Merge(f),
	}
}

func (r *Deployment) FindByID(ctx context.Context, id id.DeploymentID) (*deployment.Deployment, error) {
	return r.findOne(ctx, bson.M{
		"id": id.String(),
	}, true)
}

func (r *Deployment) FindByIDs(ctx context.Context, ids id.DeploymentIDList) ([]*deployment.Deployment, error) {
	filter := bson.M{
		"id": bson.M{
			"$in": ids.Strings(),
		},
	}
	res, err := r.find(ctx, filter)
	if err != nil {
		return nil, err
	}
	return filterDeployments(ids, res), nil
}

func (r *Deployment) FindByWorkspace(ctx context.Context, workspace accountdomain.WorkspaceID, pagination *usecasex.Pagination) ([]*deployment.Deployment, *usecasex.PageInfo, error) {
	if !r.f.CanRead(workspace) {
		return nil, usecasex.EmptyPageInfo(), nil
	}
	return r.paginate(ctx, bson.M{
		"workspaceid": workspace.String(),
	}, pagination)
}

func (r *Deployment) FindByProject(ctx context.Context, project id.ProjectID) (*deployment.Deployment, error) {
	return r.findOne(ctx, bson.M{
		"projectid": project.String(),
	}, true)
}

func (r *Deployment) Save(ctx context.Context, deployment *deployment.Deployment) error {
	if !r.f.CanWrite(deployment.Workspace()) {
		return repo.ErrOperationDenied
	}
	doc, id := mongodoc.NewDeployment(deployment)
	return r.client.SaveOne(ctx, id, doc)
}

func (r *Deployment) Remove(ctx context.Context, id id.DeploymentID) error {
	return r.client.RemoveOne(ctx, r.writeFilter(bson.M{"id": id.String()}))
}

func (r *Deployment) find(ctx context.Context, filter interface{}) ([]*deployment.Deployment, error) {
	c := mongodoc.NewDeploymentConsumer(r.f.Readable)
	if err := r.client.Find(ctx, filter, c); err != nil {
		return nil, err
	}
	return c.Result, nil
}

func (r *Deployment) findOne(ctx context.Context, filter any, filterByWorkspaces bool) (*deployment.Deployment, error) {
	var f []accountdomain.WorkspaceID
	if filterByWorkspaces {
		f = r.f.Readable
	}
	c := mongodoc.NewDeploymentConsumer(f)
	if err := r.client.FindOne(ctx, filter, c); err != nil {
		return nil, err
	}
	return c.Result[0], nil
}

func (r *Deployment) paginate(ctx context.Context, filter bson.M, pagination *usecasex.Pagination) ([]*deployment.Deployment, *usecasex.PageInfo, error) {
	c := mongodoc.NewDeploymentConsumer(r.f.Readable)
	pageInfo, err := r.client.Paginate(ctx, filter, nil, pagination, c)
	if err != nil {
		return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
	}
	return c.Result, pageInfo, nil
}

func filterDeployments(ids []id.DeploymentID, rows []*deployment.Deployment) []*deployment.Deployment {
	res := make([]*deployment.Deployment, 0, len(ids))
	for _, id := range ids {
		var r2 *deployment.Deployment
		for _, r := range rows {
			if r.ID() == id {
				r2 = r
				break
			}
		}
		res = append(res, r2)
	}
	return res
}

func (r *Deployment) writeFilter(filter interface{}) interface{} {
	return applyWorkspaceFilter(filter, r.f.Writable)
}
