package interactor

import (
	"bytes"
	"context"
	"io"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/fs"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/reearth/reearthx/account/accountinfrastructure/accountmemory"
	"github.com/reearth/reearthx/account/accountusecase"
	"github.com/spf13/afero"
	"github.com/stretchr/testify/assert"
)

func TestWorkflow_Create(t *testing.T) {
	ctx := context.Background()
	wid := workflow.NewID()
	defer workflow.MockNewID(wid)()

	ws := workspace.New().NewID().MustBuild()
	p := project.New().NewID().MustBuild()

	mfs := afero.NewMemMapFs()
	f, _ := fs.NewFile(mfs, "")
	uc := &Workflow{
		repos: &repo.Container{
			Workflow:  memory.NewWorkflow(),
			Workspace: accountmemory.NewWorkspaceWith(ws),
		},
		gateways: &gateway.Container{
			File: f,
		},
	}

	buf := bytes.NewBufferString("Hello")
	buflen := int64(buf.Len())
	res, err := uc.Create(ctx, interfaces.CreateWorkflowParam{
		WorkspaceID: ws.ID(),
		ProjectID:   p.ID(),
		Workflow: &file.File{
			Content:     io.NopCloser(buf),
			Path:        "hoge.txt",
			ContentType: "",
			Size:        buflen,
		},
	}, &usecase.Operator{
		AcOperator: &accountusecase.Operator{
			WritableWorkspaces: accountdomain.WorkspaceIDList{ws.ID()},
		},
	})
	assert.NoError(t, err)

	want := workflow.NewWorkflow(wid, p.ID(), ws.ID(), "hoge.txt")

	assert.NoError(t, err)
	assert.Equal(t, want, res)
	w, _ := uc.repos.Workflow.FindByID(ctx, wid)
	assert.Equal(t, want, w)
}
