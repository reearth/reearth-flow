package gql

import (
	"context"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/interactor"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/rerror"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// perWorkspaceChecker allows/denies CheckPermission per workspace, so a test
// can simulate a batch spanning an authorized and an unauthorized workspace.
type perWorkspaceChecker struct {
	allowed map[accountsid.WorkspaceID]bool
}

func (c *perWorkspaceChecker) CheckPermission(_ context.Context, _, _ string, workspaceID ...accountsid.WorkspaceID) (bool, error) {
	if len(workspaceID) == 0 {
		return false, nil
	}
	return c.allowed[workspaceID[0]], nil
}

// stubParameterUsecase returns canned results from FetchByProject; other
// methods panic if called since these tests only exercise that path.
type stubParameterUsecase struct {
	interfaces.Parameter
	result *parameter.ParameterList
	err    error
}

func (s *stubParameterUsecase) FetchByProject(_ context.Context, _ id.ProjectID) (*parameter.ParameterList, error) {
	return s.result, s.err
}

// Query.parameters is a root field with no already-authorized parent, so it
// must surface FetchByProject's error rather than collapsing it into an
// empty, successful list the way the batched Project.parameters path does.
func TestQueryResolver_Parameters_DeniedCallerReturnsError(t *testing.T) {
	r := &queryResolver{}
	loaders := &Loaders{Parameter: NewParameterLoader(&stubParameterUsecase{err: interfaces.ErrOperationDenied})}
	ctx := context.WithValue(context.Background(), contextLoaders, loaders)

	res, err := r.Parameters(ctx, gqlmodel.ID(id.NewProjectID().String()))

	require.Error(t, err)
	assert.ErrorIs(t, err, interfaces.ErrOperationDenied)
	assert.Nil(t, res)
}

func TestQueryResolver_Parameters_NonexistentProjectReturnsError(t *testing.T) {
	r := &queryResolver{}
	loaders := &Loaders{Parameter: NewParameterLoader(&stubParameterUsecase{err: rerror.ErrNotFound})}
	ctx := context.WithValue(context.Background(), contextLoaders, loaders)

	res, err := r.Parameters(ctx, gqlmodel.ID(id.NewProjectID().String()))

	require.Error(t, err)
	assert.ErrorIs(t, err, rerror.ErrNotFound)
	assert.Nil(t, res)
}

func TestQueryResolver_Parameters_Success(t *testing.T) {
	pid := id.NewProjectID()
	param, err := parameter.New().ProjectID(pid).Name("p").Type(parameter.TypeText).Build()
	require.NoError(t, err)
	list := parameter.NewParameterList([]*parameter.Parameter{param})

	r := &queryResolver{}
	loaders := &Loaders{Parameter: NewParameterLoader(&stubParameterUsecase{result: list})}
	ctx := context.WithValue(context.Background(), contextLoaders, loaders)

	res, err := r.Parameters(ctx, gqlmodel.ID(pid.String()))

	require.NoError(t, err)
	assert.Len(t, res, 1)
}

// TestParameterLoader_Fetch_ReversedOrder_DeniedFirstDoesNotShiftAllowed is a
// loader-level regression test: ParameterLoader is a positional dataloader
// (gqldataloader maps response slot i back to the i-th requested key), so
// Parameter.Fetch must return one slot per requested id, nil for a denied
// one, rather than compacting to just the allowed parameters. Requesting the
// denied id FIRST and the allowed id second is the case that would have
// caught the bug: compacting to [allowed] would make the loader cache the
// allowed parameter under the denied id's key and return nil for the id the
// caller actually has access to.
func TestParameterLoader_Fetch_ReversedOrder_DeniedFirstDoesNotShiftAllowed(t *testing.T) {
	wsAllowed := accountsid.NewWorkspaceID()
	wsDenied := accountsid.NewWorkspaceID()
	projectRepo := memory.NewProject()
	paramRepo := memory.NewParameter()

	deniedProject := project.New().NewID().Workspace(wsDenied).Name("victim").MustBuild()
	require.NoError(t, projectRepo.Save(context.Background(), deniedProject))
	deniedParam, err := parameter.New().ProjectID(deniedProject.ID()).Name("denied").Type(parameter.TypeText).Build()
	require.NoError(t, err)
	require.NoError(t, paramRepo.Save(context.Background(), deniedParam))

	allowedProject := project.New().NewID().Workspace(wsAllowed).Name("own").MustBuild()
	require.NoError(t, projectRepo.Save(context.Background(), allowedProject))
	allowedParam, err := parameter.New().ProjectID(allowedProject.ID()).Name("allowed").Type(parameter.TypeText).Build()
	require.NoError(t, err)
	require.NoError(t, paramRepo.Save(context.Background(), allowedParam))

	checker := &perWorkspaceChecker{allowed: map[accountsid.WorkspaceID]bool{wsAllowed: true, wsDenied: false}}
	usecase := interactor.NewParameter(&repo.Container{Parameter: paramRepo, Project: projectRepo}, checker)
	loader := NewParameterLoader(usecase)

	res, errs := loader.Fetch(context.Background(), []gqlmodel.ID{
		gqlmodel.ID(deniedParam.ID().String()),
		gqlmodel.ID(allowedParam.ID().String()),
	})

	require.Empty(t, errs)
	require.Len(t, res, 2, "one slot per requested id")
	assert.Nil(t, res[0], "the denied id's slot must be nil, not the allowed parameter")
	require.NotNil(t, res[1], "the allowed id's slot must hold its own parameter")
	assert.Equal(t, allowedParam.ID().String(), string(res[1].ID))
}
