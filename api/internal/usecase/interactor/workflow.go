package interactor

import (
	"context"
	"errors"
	"net/url"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
)

var (
	ErrEmptyWorkspaceID = errors.New("require workspace id")
	ErrEmptyURL         = errors.New("require valid url")
	ErrEmptySize        = errors.New("file size cannot be zero")
)

type Workflow struct {
	repos    *repo.Container
	gateways *gateway.Container
}

func NewWorkflow(r *repo.Container, g *gateway.Container) interfaces.Workflow {
	return &Workflow{
		repos:    r,
		gateways: g,
	}
}

func (i *Workflow) Fetch(ctx context.Context, id id.WorkflowID) (*workflow.Workflow, error) {
	return i.repos.Workflow.FindByID(ctx, id)
}

func (i *Workflow) Create(ctx context.Context, p interfaces.CreateWorkflowParam, operator *usecase.Operator) (*workflow.Workflow, error) {
	if p.Workflow == nil {
		return nil, interfaces.ErrFileNotIncluded
	}

	ws, err := i.repos.Workspace.FindByID(ctx, p.WorkspaceID)
	if err != nil {
		return nil, err
	}

	if !operator.IsWritableWorkspace(ws.ID()) {
		return nil, interfaces.ErrOperationDenied
	}

	wID := id.NewWorkflowID()

	url, err := i.gateways.Workflow.UploadWorkflow(ctx, p.Workflow)

	w := workflow.NewWorkflow(wID, p.ProjectID, p.WorkspaceID, url.String())

	if err := i.repos.Workflow.Save(ctx, w); err != nil {
		return nil, err
	}

	return w, nil
}

func (i *Workflow) Remove(ctx context.Context, wid id.WorkflowID, operator *usecase.Operator) (result id.WorkflowID, err error) {
	return Run1(
		ctx, operator, i.repos,
		Usecase().Transaction(),
		func(ctx context.Context) (id.WorkflowID, error) {
			workflow, err := i.repos.Workflow.FindByID(ctx, wid)
			if err != nil {
				return wid, err
			}

			if ok := operator.IsWritableWorkspace(workflow.Workspace); !ok {
				return wid, interfaces.ErrOperationDenied
			}

			if url, _ := url.Parse(workflow.URL); url != nil {
				if err := i.gateways.File.RemoveAsset(ctx, url); err != nil {
					return wid, err
				}
			}

			return wid, i.repos.Workflow.Remove(ctx, wid)
		},
	)
}
