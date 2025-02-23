package mongo

import (
	"context"
	"fmt"
	"strconv"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/rerror"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo/options"
)

var (
	deploymentIndexes       = []string{"workspaceid"}
	deploymentUniqueIndexes = []string{"id"}
)

type Deployment struct {
	client *mongox.Collection
	f      repo.WorkspaceFilter
}

type DeploymentAdapter struct {
	*Deployment
}

func NewDeployment(client *mongox.Client) repo.Deployment {
	return &DeploymentAdapter{
		Deployment: &Deployment{
			client: client.WithCollection("deployment"),
		},
	}
}

func (r *Deployment) Init(ctx context.Context) error {
	return createIndexes(ctx, r.client, deploymentIndexes, deploymentUniqueIndexes)
}

func (a *DeploymentAdapter) Filtered(f repo.WorkspaceFilter) repo.Deployment {
	return &DeploymentAdapter{
		Deployment: &Deployment{
			client: a.client,
			f:      a.f.Merge(f),
		},
	}
}

func (r *Deployment) FindByID(ctx context.Context, id id.DeploymentID) (*deployment.Deployment, error) {
	return r.findOne(ctx, bson.M{
		"id": id.String(),
	}, true)
}

func (r *Deployment) FindByIDs(ctx context.Context, ids id.DeploymentIDList) ([]*deployment.Deployment, error) {
	if len(ids) == 0 {
		return nil, nil
	}

	filter := bson.M{
		"id": bson.M{
			"$in": ids.Strings(),
		},
	}

	c := mongodoc.NewDeploymentConsumer(r.f.Readable)
	if err := r.client.Find(ctx, filter, c); err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, err)
	}

	return filterDeployments(ids, c.Result), nil
}

func (r *DeploymentAdapter) FindByWorkspace(ctx context.Context, id accountdomain.WorkspaceID, pagination *interfaces.PaginationParam) ([]*deployment.Deployment, *interfaces.PageBasedInfo, error) {
	if !r.f.CanRead(id) {
		return nil, interfaces.NewPageBasedInfo(0, 1, 1), nil
	}

	c := mongodoc.NewDeploymentConsumer(r.f.Readable)
	filter := bson.M{"workspaceid": id.String()}

	if pagination != nil && pagination.Page != nil {
		skip := int64((pagination.Page.Page - 1) * pagination.Page.PageSize)
		limit := int64(pagination.Page.PageSize)

		total, err := r.client.Count(ctx, filter)
		if err != nil {
			return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
		}

		sort := bson.D{{Key: "updatedat", Value: -1}}

		if pagination.Page.OrderBy != nil {
			sortDir := -1
			if pagination.Page.OrderDir != nil && *pagination.Page.OrderDir == "ASC" {
				sortDir = 1
			}

			fieldNameMap := map[string]string{
				"updatedAt":   "updatedat",
				"description": "description",
				"version":     "version",
				"id":          "id",
			}

			fieldName := *pagination.Page.OrderBy
			if mongoField, ok := fieldNameMap[fieldName]; ok {
				fieldName = mongoField
			}
			sort = bson.D{{Key: fieldName, Value: sortDir}}
		}

		opts := options.Find().SetSkip(skip).SetLimit(limit).SetSort(sort)
		if err := r.client.Find(ctx, filter, c, opts); err != nil {
			return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
		}

		return c.Result, interfaces.NewPageBasedInfo(total, pagination.Page.Page, pagination.Page.PageSize), nil
	}

	if err := r.client.Find(ctx, filter, c); err != nil {
		return nil, nil, rerror.ErrInternalByWithContext(ctx, err)
	}
	total := int64(len(c.Result))
	return c.Result, interfaces.NewPageBasedInfo(total, 1, len(c.Result)), nil
}

func (r *Deployment) FindByProject(ctx context.Context, pid id.ProjectID) (*deployment.Deployment, error) {
	return r.findOne(ctx, bson.M{
		"projectid": pid.String(),
		"ishead":    true,
	}, true)
}

func (r *Deployment) FindByVersion(ctx context.Context, wsID accountdomain.WorkspaceID, pID *id.ProjectID, version string) (*deployment.Deployment, error) {
	filter := bson.M{
		"workspaceid": wsID.String(),
		"version":     version,
	}
	if pID != nil {
		filter["projectid"] = pID.String()
	}
	return r.findOne(ctx, filter, true)
}

func (r *Deployment) FindHead(ctx context.Context, wsID accountdomain.WorkspaceID, pID *id.ProjectID) (*deployment.Deployment, error) {
	filter := bson.M{
		"workspaceid": wsID.String(),
		"ishead":      true,
	}
	if pID != nil {
		filter["projectid"] = pID.String()
	}
	return r.findOne(ctx, filter, true)
}

func (r *Deployment) FindVersions(ctx context.Context, wsID accountdomain.WorkspaceID, pID *id.ProjectID) ([]*deployment.Deployment, error) {
	filter := bson.M{
		"workspaceid": wsID.String(),
	}
	if pID != nil {
		filter["projectid"] = pID.String()
	}

	c := mongodoc.NewDeploymentConsumer(r.f.Readable)
	if err := r.client.Find(ctx, filter, c); err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, err)
	}
	return c.Result, nil
}

func (r *Deployment) Create(ctx context.Context, param interfaces.CreateDeploymentParam) (*deployment.Deployment, error) {
	d := deployment.New().
		NewID().
		Workspace(param.Workspace).
		Project(param.Project).
		Description(param.Description).
		WorkflowURL(param.Workflow.Path)

	if param.Project != nil {
		head, _ := r.FindHead(ctx, param.Workspace, param.Project)

		d = d.IsHead(true)
		if head != nil {
			currentHeadID := head.ID()
			d = d.HeadID(&currentHeadID)
			d = d.Version(incrementVersion(head.Version()))

			head.SetIsHead(false)
			if err := r.Save(ctx, head); err != nil {
				return nil, err
			}
		} else {
			d = d.Version("v1")
		}
	} else {
		d = d.Version("v0")
		d = d.IsHead(false)
	}

	dep := d.MustBuild()
	if err := r.Save(ctx, dep); err != nil {
		return nil, err
	}

	return dep, nil
}

func incrementVersion(version string) string {
	if len(version) < 2 || version[0] != 'v' {
		return "v1"
	}
	num := version[1:]
	if n, err := strconv.Atoi(num); err == nil {
		return fmt.Sprintf("v%d", n+1)
	}
	return "v1"
}

func (r *Deployment) Update(ctx context.Context, param interfaces.UpdateDeploymentParam) (*deployment.Deployment, error) {
	d, err := r.FindByID(ctx, param.ID)
	if err != nil {
		return nil, err
	}

	if param.Description != nil {
		d.SetDescription(*param.Description)
	}

	if param.Workflow != nil {
		d.SetWorkflowURL(param.Workflow.Path)
	}

	if err := r.Save(ctx, d); err != nil {
		return nil, err
	}

	return d, nil
}

func (r *Deployment) Delete(ctx context.Context, id id.DeploymentID) error {
	return r.Remove(ctx, id)
}

func (r *Deployment) Fetch(ctx context.Context, ids []id.DeploymentID) ([]*deployment.Deployment, error) {
	return r.FindByIDs(ctx, ids)
}

func (r *Deployment) Save(ctx context.Context, deployment *deployment.Deployment) error {
	if !r.f.CanWrite(deployment.Workspace()) {
		return interfaces.ErrOperationDenied
	}

	doc, err := mongodoc.NewDeployment(deployment)
	if err != nil {
		return rerror.ErrInternalByWithContext(ctx, err)
	}

	return r.client.SaveOne(ctx, doc.ID, doc)
}

func (r *Deployment) Remove(ctx context.Context, id id.DeploymentID) error {
	return r.client.RemoveOne(ctx, bson.M{
		"id": id.String(),
	})
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
