package mongo

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/mongox"
	"go.mongodb.org/mongo-driver/bson"
)

var (
	workflowIndexes       = []string{"workspace"}
	workflowUniqueIndexes = []string{"id"}
)

type Workflow struct {
	client *mongox.ClientCollection
	f      repo.WorkspaceFilter
}

func NewWorkflow(client *mongox.Client) *Workflow {
	return &Workflow{client: client.WithCollection("workflow")}
}

func (r *Workflow) Init(ctx context.Context) error {
	return createIndexes(ctx, r.client, workflowIndexes, workflowUniqueIndexes)
}

func (r *Workflow) Filtered(f repo.WorkspaceFilter) repo.Workflow {
	return &Workflow{
		client: r.client,
		f:      r.f.Merge(f),
	}
}

func (r *Workflow) FindByID(ctx context.Context, id id.WorkflowID) (*workflow.Workflow, error) {
	return r.findOne(ctx, bson.M{
		"id": id.String(),
	}, true)
}

func (r *Workflow) Save(ctx context.Context, workflow *workflow.Workflow) error {
	if !r.f.CanWrite(workflow.Workspace()) {
		return repo.ErrOperationDenied
	}
	doc, id := mongodoc.NewWorkflow(workflow)
	return r.client.SaveOne(ctx, id, doc)
}

func (r *Workflow) findOne(ctx context.Context, filter any, filterByWorkspaces bool) (*workflow.Workflow, error) {
	var f []accountdomain.WorkspaceID
	if filterByWorkspaces {
		f = r.f.Readable
	}
	c := mongodoc.NewWorkflowConsumer(f)
	if err := r.client.FindOne(ctx, filter, c); err != nil {
		return nil, err
	}
	return c.Result[0], nil
}

func (r *Workflow) Remove(ctx context.Context, id id.WorkflowID) error {
	return r.client.RemoveOne(ctx, r.writeFilter(bson.M{
		"id": id.String(),
	}))
}

func (r *Workflow) writeFilter(filter any) any {
	return applyWorkspaceFilter(filter, r.f.Writable)
}
