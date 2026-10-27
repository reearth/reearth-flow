package gql

import (
	"context"
	"errors"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/samber/lo"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// stubProjectUsecase records the arguments FindByWorkspace receives and
// returns canned results; other methods panic if called.
type stubProjectUsecase struct {
	interfaces.Project
	err             error
	info            *interfaces.PageBasedInfo
	pagination      *interfaces.PaginationParam
	keyword         *string
	includeArchived *bool
	result          []*project.Project
	createdBy       accountsid.UserIDList
	workspace       accountsid.WorkspaceID
	called          bool
}

func (s *stubProjectUsecase) FindByWorkspace(_ context.Context, wid accountsid.WorkspaceID, p *interfaces.PaginationParam, keyword *string, includeArchived *bool, createdBy accountsid.UserIDList) ([]*project.Project, *interfaces.PageBasedInfo, error) {
	s.called = true
	s.workspace = wid
	s.pagination = p
	s.keyword = keyword
	s.includeArchived = includeArchived
	s.createdBy = createdBy
	return s.result, s.info, s.err
}

func TestProjectLoader_FindByWorkspacePage(t *testing.T) {
	wid := accountsid.NewWorkspaceID()
	creator, other := accountsid.NewUserID(), accountsid.NewUserID()
	p1 := project.New().ID(id.NewProjectID()).Workspace(wid).Name("a").CreatedBy(&creator).MustBuild()
	p2 := project.New().ID(id.NewProjectID()).Workspace(wid).Name("b").CreatedBy(&other).MustBuild()

	uc := &stubProjectUsecase{
		result: []*project.Project{p1, p2},
		info:   interfaces.NewPageBasedInfo(12, 2, 5),
	}
	l := NewProjectLoader(uc)

	desc := gqlmodel.OrderDirectionDesc
	pagination := gqlmodel.PageBasedPagination{
		Page:     2,
		PageSize: 5,
		OrderBy:  lo.ToPtr("updatedAt"),
		OrderDir: &desc,
	}

	res, err := l.FindByWorkspacePage(
		context.Background(),
		gqlmodel.IDFrom(wid),
		lo.ToPtr("kw"),
		lo.ToPtr(true),
		[]gqlmodel.ID{gqlmodel.IDFrom(creator), gqlmodel.IDFrom(other)},
		pagination,
	)
	require.NoError(t, err)

	// Arguments reach the usecase converted to domain types.
	require.True(t, uc.called)
	assert.Equal(t, wid, uc.workspace)
	assert.Equal(t, lo.ToPtr("kw"), uc.keyword)
	assert.Equal(t, lo.ToPtr(true), uc.includeArchived)
	assert.Equal(t, accountsid.UserIDList{creator, other}, uc.createdBy)
	require.NotNil(t, uc.pagination)
	require.NotNil(t, uc.pagination.Page)
	assert.Equal(t, 2, uc.pagination.Page.Page)
	assert.Equal(t, 5, uc.pagination.Page.PageSize)
	assert.Equal(t, lo.ToPtr("updatedAt"), uc.pagination.Page.OrderBy)
	assert.Equal(t, lo.ToPtr("DESC"), uc.pagination.Page.OrderDir)

	// Results are mapped into the connection in order.
	require.Len(t, res.Nodes, 2)
	assert.Equal(t, gqlmodel.IDFrom(p1.ID()), res.Nodes[0].ID)
	assert.Equal(t, gqlmodel.IDFrom(p2.ID()), res.Nodes[1].ID)
	assert.Equal(t, lo.ToPtr(gqlmodel.IDFrom(creator)), res.Nodes[0].CreatedByID)
	assert.Equal(t, 12, res.TotalCount)
	require.NotNil(t, res.PageInfo)
	assert.Equal(t, 12, res.PageInfo.TotalCount)
	assert.Equal(t, lo.ToPtr(2), res.PageInfo.CurrentPage)
	assert.Equal(t, lo.ToPtr(3), res.PageInfo.TotalPages)
}

func TestProjectLoader_FindByWorkspacePage_NoCreatedByFilter(t *testing.T) {
	uc := &stubProjectUsecase{info: interfaces.NewPageBasedInfo(0, 1, 10)}
	l := NewProjectLoader(uc)

	res, err := l.FindByWorkspacePage(
		context.Background(),
		gqlmodel.IDFrom(accountsid.NewWorkspaceID()),
		nil, nil, nil,
		gqlmodel.PageBasedPagination{Page: 1, PageSize: 10},
	)
	require.NoError(t, err)

	require.True(t, uc.called)
	assert.Nil(t, uc.createdBy)
	assert.Nil(t, uc.keyword)
	assert.Nil(t, uc.includeArchived)
	assert.Empty(t, res.Nodes)
	assert.NotNil(t, res.Nodes, "an empty page serializes as [] rather than null")
	assert.Equal(t, 0, res.TotalCount)
}

func TestProjectLoader_FindByWorkspacePage_InvalidWorkspaceID(t *testing.T) {
	uc := &stubProjectUsecase{}
	l := NewProjectLoader(uc)

	res, err := l.FindByWorkspacePage(
		context.Background(),
		gqlmodel.ID("not-an-id"),
		nil, nil, nil,
		gqlmodel.PageBasedPagination{Page: 1, PageSize: 10},
	)
	require.Error(t, err)
	assert.Nil(t, res)
	assert.False(t, uc.called)
}

// A malformed creator ID must fail the query rather than be dropped, since
// dropping it would widen the filter (possibly to every project).
func TestProjectLoader_FindByWorkspacePage_InvalidCreatedByID(t *testing.T) {
	uc := &stubProjectUsecase{}
	l := NewProjectLoader(uc)

	res, err := l.FindByWorkspacePage(
		context.Background(),
		gqlmodel.IDFrom(accountsid.NewWorkspaceID()),
		nil, nil,
		[]gqlmodel.ID{gqlmodel.IDFrom(accountsid.NewUserID()), "not-an-id"},
		gqlmodel.PageBasedPagination{Page: 1, PageSize: 10},
	)
	require.Error(t, err)
	assert.Nil(t, res)
	assert.False(t, uc.called)
}

func TestProjectLoader_FindByWorkspacePage_UsecaseError(t *testing.T) {
	wantErr := errors.New("boom")
	uc := &stubProjectUsecase{err: wantErr}
	l := NewProjectLoader(uc)

	res, err := l.FindByWorkspacePage(
		context.Background(),
		gqlmodel.IDFrom(accountsid.NewWorkspaceID()),
		nil, nil, nil,
		gqlmodel.PageBasedPagination{Page: 1, PageSize: 10},
	)
	assert.ErrorIs(t, err, wantErr)
	assert.Nil(t, res)
}
