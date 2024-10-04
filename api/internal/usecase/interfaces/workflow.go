package interfaces

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
	"github.com/reearth/reearthx/account/accountdomain"
)

type CreateWorkflowParam struct {
	WorkspaceID accountdomain.WorkspaceID
	ProjectID   id.ProjectID
	Workflow    *file.File
}

type Workflow interface {
	Fetch(context.Context, id.WorkflowID) (*workflow.Workflow, error)
	Create(context.Context, CreateWorkflowParam, *usecase.Operator) (*workflow.Workflow, error)
	Remove(context.Context, id.WorkflowID, *usecase.Operator) (id.WorkflowID, error)
}
