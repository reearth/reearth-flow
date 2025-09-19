package mongodoc

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
	"golang.org/x/exp/slices"
)

type WorkflowDocument struct {
	ID        string
	Project   string
	Workspace string
	URL       string
}

type WorkflowConsumer = Consumer[*WorkflowDocument, *workflow.Workflow]

func NewWorkflowConsumer(workspaces []id.WorkspaceID) *WorkflowConsumer {
	return NewConsumer[*WorkflowDocument, *workflow.Workflow](func(a *workflow.Workflow) bool {
		return workspaces == nil || slices.Contains(workspaces, a.Workspace())
	})
}

func NewWorkflow(wf *workflow.Workflow) (*WorkflowDocument, string) {
	wid := wf.ID().String()
	return &WorkflowDocument{
		ID:        wid,
		Project:   wf.Project().String(),
		Workspace: wf.Workspace().String(),
		URL:       wf.URL(),
	}, wid
}

func (d *WorkflowDocument) Model() (*workflow.Workflow, error) {
	wid, err := id.WorkflowIDFrom(d.ID)
	if err != nil {
		return nil, err
	}
	pid, err := id.ProjectIDFrom(d.Project)
	if err != nil {
		return nil, err
	}
	tid, err := id.WorkspaceIDFrom(d.Workspace)
	if err != nil {
		return nil, err
	}
	wf := workflow.NewWorkflow(
		wid,
		pid,
		tid,
		d.URL,
	)

	return wf, nil
}
