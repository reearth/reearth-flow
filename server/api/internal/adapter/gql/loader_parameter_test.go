package gql

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearthx/rerror"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

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
