package repo

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
	"github.com/reearth/reearthx/account/accountdomain"
)

type Workflow interface {
	Filtered(WorkspaceFilter) Workflow
	FindByID(context.Context, accountdomain.WorkspaceID, id.WorkflowID) (*workflow.Workflow, error)
	Save(context.Context, accountdomain.WorkspaceID, *workflow.Workflow) error
	Remove(context.Context, accountdomain.WorkspaceID, id.WorkflowID) error
}
