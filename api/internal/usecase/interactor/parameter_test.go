package interactor

import (
	"context"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/reearth/reearthx/account/accountinfrastructure/accountmemory"
	"github.com/reearth/reearthx/account/accountusecase"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
	"github.com/stretchr/testify/assert"
)

func setupParameterInteractor() (interfaces.Parameter, context.Context, *repo.Container, id.ProjectID, *usecase.Operator) {
	ctx := context.Background()

	// Create memory-based repositories for testing
	paramRepo := memory.NewParameter()
	projectRepo := memory.NewProject()
	workspaceRepo := accountmemory.NewWorkspace()

	r := &repo.Container{
		Parameter:   paramRepo,
		Project:     projectRepo,
		Transaction: &usecasex.NopTransaction{},
	}

	// Set up a workspace, project and operator
	ws := workspace.New().NewID().MustBuild()
	_ = workspaceRepo.Save(ctx, ws)

	// We'll need a project
	pid := project.NewID()
	defer project.MockNewID(pid)()
	prj := project.New().ID(pid).Workspace(ws.ID()).Name("testproject").UpdatedAt(time.Now()).MustBuild()
	_ = projectRepo.Save(ctx, prj)

	// Operator with write access to the project's workspace
	op := &usecase.Operator{
		AcOperator: &accountusecase.Operator{
			WritableWorkspaces: workspace.IDList{ws.ID()},
		},
	}

	i := NewParameter(r)
	return i, ctx, r, pid, op
}

func TestParameter_DeclareParameter(t *testing.T) {
	i, ctx, _, pid, op := setupParameterInteractor()

	// Declare a parameter
	name := "param1"
	typ := parameter.TypeText
	val := "initial value"
	req := true
	p, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID: pid,
		Name:      name,
		Type:      typ,
		Required:  req,
		Value:     val,
		Index:     nil, // let it auto-determine
	}, op)
	assert.NoError(t, err)
	assert.NotNil(t, p)
	assert.Equal(t, name, p.Name())
	assert.Equal(t, typ, p.Type())
	assert.Equal(t, req, p.Required())
	assert.Equal(t, val, p.Value())
	assert.Equal(t, 0, p.Index()) // first parameter gets index 0
}

func TestParameter_DeclareParameter_NonexistentProject(t *testing.T) {
	i, ctx, _, _, op := setupParameterInteractor()

	// Use a random project ID that doesn't exist
	nonexistentPID := id.NewProjectID()
	p, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID: nonexistentPID,
		Name:      "param2",
		Type:      parameter.TypeNumber,
		Value:     123,
	}, op)
	assert.Nil(t, p)
	assert.Same(t, rerror.ErrNotFound, err)
}

func TestParameter_Fetch(t *testing.T) {
	i, ctx, _, pid, op := setupParameterInteractor()

	// Create a few parameters
	p1, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID: pid,
		Name:      "param1",
		Type:      parameter.TypeText,
		Value:     "val1",
	}, op)
	assert.NoError(t, err)

	p2, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID: pid,
		Name:      "param2",
		Type:      parameter.TypeNumber,
		Value:     42,
	}, op)
	assert.NoError(t, err)

	params, err := i.Fetch(ctx, id.ParameterIDList{p1.ID(), p2.ID()}, op)
	assert.NoError(t, err)
	assert.NotNil(t, params)
	assert.Len(t, *params, 2)
}

func TestParameter_FetchByProject(t *testing.T) {
	i, ctx, _, pid, op := setupParameterInteractor()

	// Create parameters under the project
	_, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID: pid,
		Name:      "param1",
		Type:      parameter.TypeText,
		Value:     "val1",
	}, op)
	assert.NoError(t, err)

	_, err = i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID: pid,
		Name:      "param2",
		Type:      parameter.TypeNumber,
		Value:     100,
	}, op)
	assert.NoError(t, err)

	params, err := i.FetchByProject(ctx, pid, op)
	assert.NoError(t, err)
	assert.NotNil(t, params)
	assert.Len(t, *params, 2)
}

func TestParameter_RemoveParameter(t *testing.T) {
	i, ctx, _, pid, op := setupParameterInteractor()

	// Create parameters for removal test
	p1, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID: pid,
		Name:      "param1",
		Type:      parameter.TypeText,
		Value:     "val1",
	}, op)
	assert.NoError(t, err)

	p2, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID: pid,
		Name:      "param2",
		Type:      parameter.TypeText,
		Value:     "val2",
	}, op)
	assert.NoError(t, err)

	// Remove param1
	removedID, err := i.RemoveParameter(ctx, p1.ID(), op)
	assert.NoError(t, err)
	assert.Equal(t, p1.ID(), removedID)

	// Check that only p2 remains, and its index has been updated if needed
	params, err := i.FetchByProject(ctx, pid, op)
	assert.NoError(t, err)
	assert.Len(t, *params, 1)
	assert.Equal(t, p2.ID(), (*params)[0].ID())
	assert.Equal(t, 0, (*params)[0].Index())
}

func TestParameter_UpdateParameterOrder(t *testing.T) {
	i, ctx, _, pid, op := setupParameterInteractor()

	// Create parameters in some order: param1 (0), param2 (1), param3 (2)
	p1, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID: pid,
		Name:      "param1",
		Type:      parameter.TypeText,
		Value:     "val1",
	}, op)
	assert.NoError(t, err)

	p2, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID: pid,
		Name:      "param2",
		Type:      parameter.TypeText,
		Value:     "val2",
	}, op)
	assert.NoError(t, err)

	p3, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID: pid,
		Name:      "param3",
		Type:      parameter.TypeText,
		Value:     "val3",
	}, op)
	assert.NoError(t, err)

	// Move param3 to the front (index 0)
	updatedParams, err := i.UpdateParameterOrder(ctx, interfaces.UpdateParameterOrderParam{
		ProjectID: pid,
		ParamID:   p3.ID(),
		NewIndex:  0,
	}, op)
	assert.NoError(t, err)
	assert.NotNil(t, updatedParams)

	// Check new order: p3(0), p1(1), p2(2)
	p3u := updatedParams.FindByID(p3.ID())
	p1u := updatedParams.FindByID(p1.ID())
	p2u := updatedParams.FindByID(p2.ID())
	assert.NotNil(t, p3u)
	assert.NotNil(t, p1u)
	assert.NotNil(t, p2u)
	assert.Equal(t, 0, p3u.Index())
	assert.Equal(t, 1, p1u.Index())
	assert.Equal(t, 2, p2u.Index())
}

func TestParameter_UpdateParameterValue(t *testing.T) {
	i, ctx, _, pid, op := setupParameterInteractor()

	p1, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID: pid,
		Name:      "param1",
		Type:      parameter.TypeText,
		Value:     "old value",
	}, op)
	assert.NoError(t, err)

	// Update param1's value
	newVal := "new value"
	updatedParam, err := i.UpdateParameterValue(ctx, interfaces.UpdateParameterValueParam{
		ParamID: p1.ID(),
		Value:   newVal,
	}, op)
	assert.NoError(t, err)
	assert.NotNil(t, updatedParam)
	assert.Equal(t, newVal, updatedParam.Value())
}

func TestParameter_UpdateParameterValue_NotFound(t *testing.T) {
	i, ctx, _, _, op := setupParameterInteractor()

	// Try updating a parameter that does not exist
	nonexistentParamID := id.NewParameterID()
	updatedParam, err := i.UpdateParameterValue(ctx, interfaces.UpdateParameterValueParam{
		ParamID: nonexistentParamID,
		Value:   "something",
	}, op)
	assert.Nil(t, updatedParam)
	assert.Same(t, rerror.ErrNotFound, err)
}
