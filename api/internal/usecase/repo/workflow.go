package repo

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
)

type Workflow interface {
	Filtered(WorkspaceFilter) Workflow
	FindByID(context.Context, id.WorkflowID) (*workflow.Workflow, error)
	Save(context.Context, *workflow.Workflow) error
	Remove(context.Context, id.WorkflowID) error
}
