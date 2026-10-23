package internalapi

import (
	"context"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	flowv1 "github.com/reearth/reearth-proto/gen/go/flow/v1"
	"github.com/reearth/reearthx/rerror"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// fakeProjects answers the project lookups and records the last search. Any
// other method panics through the nil embedded interface.
type fakeProjects struct {
	interfaces.Project
	project *project.Project
	search  interfaces.ProjectSearchParam
}

func (f *fakeProjects) FindViewable(_ context.Context, pid id.ProjectID) (*project.Project, error) {
	if f.project == nil || f.project.ID() != pid {
		return nil, rerror.ErrNotFound
	}
	return f.project, nil
}

func (f *fakeProjects) FindViewableByAlias(_ context.Context, wid accountsid.WorkspaceID, alias string) (*project.Project, error) {
	if f.project == nil || f.project.Workspace() != wid || f.project.Alias() != alias {
		return nil, rerror.ErrNotFound
	}
	return f.project, nil
}

func (f *fakeProjects) Search(_ context.Context, p interfaces.ProjectSearchParam) ([]*project.Project, *interfaces.PageBasedInfo, error) {
	f.search = p
	return []*project.Project{f.project}, interfaces.NewPageBasedInfo(1, p.Page, p.PageSize), nil
}

func withProjects(f *fakeProjects) context.Context {
	return adapter.AttachUsecases(context.Background(), &interfaces.Container{Project: f})
}

func TestParseWorkspaceID(t *testing.T) {
	wid := accountsid.NewWorkspaceID()
	got, err := parseWorkspaceID(wid.String())
	require.NoError(t, err)
	assert.Equal(t, wid, got)

	for _, in := range []string{"", "not-an-id"} {
		_, err := parseWorkspaceID(in)
		assert.Equal(t, codes.InvalidArgument, status.Code(err), in)
	}
}

func TestPageParam(t *testing.T) {
	tests := []struct {
		in                 *flowv1.PageRequest
		name               string
		wantPage, wantSize int
	}{
		{nil, "defaults", 1, interfaces.DefaultPageSize},
		{&flowv1.PageRequest{Page: 3, PageSize: 10}, "as asked", 3, 10},
		{&flowv1.PageRequest{Page: -1, PageSize: 0}, "below 1", 1, interfaces.DefaultPageSize},
		{&flowv1.PageRequest{Page: 2, PageSize: 500}, "capped", 2, interfaces.MaxPageSize},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			param, page, size := pageParam(tt.in)
			assert.Equal(t, tt.wantPage, page)
			assert.Equal(t, tt.wantSize, size)
			assert.Equal(t, tt.wantPage, param.Page.Page)
			assert.Equal(t, tt.wantSize, param.Page.PageSize)
		})
	}
}

func TestSearchProjects_Sort(t *testing.T) {
	p := project.New().NewID().Workspace(accountsid.NewWorkspaceID()).MustBuild()
	tests := []struct {
		sort    *flowv1.ProjectSort
		name    string
		wantBy  interfaces.ProjectSortField
		wantAsc bool
	}{
		{nil, "default", interfaces.ProjectSortUpdatedAt, false},
		{&flowv1.ProjectSort{Field: flowv1.ProjectSortField_PROJECT_SORT_FIELD_UPDATED_AT}, "updated", interfaces.ProjectSortUpdatedAt, false},
		{&flowv1.ProjectSort{Field: flowv1.ProjectSortField_PROJECT_SORT_FIELD_NAME, Direction: flowv1.SortDirection_SORT_DIRECTION_ASC}, "name ascending", interfaces.ProjectSortName, true},
		{&flowv1.ProjectSort{Field: flowv1.ProjectSortField_PROJECT_SORT_FIELD_STAR_COUNT, Direction: flowv1.SortDirection_SORT_DIRECTION_DESC}, "stars", interfaces.ProjectSortStarCount, false},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			f := &fakeProjects{project: p}
			projects, page, err := searchProjects(withProjects(f), interfaces.ProjectSearchParam{Keyword: "k"}, tt.sort, &flowv1.PageRequest{PageSize: 500})
			require.NoError(t, err)
			assert.Len(t, projects, 1)
			assert.Equal(t, tt.wantBy, f.search.SortBy)
			assert.Equal(t, tt.wantAsc, f.search.SortAscending)
			assert.Equal(t, "k", f.search.Keyword, "the filters are passed through")
			assert.Equal(t, interfaces.MaxPageSize, f.search.PageSize)
			assert.Equal(t, int32(interfaces.MaxPageSize), page.GetPageSize())
			assert.Equal(t, int64(1), page.GetTotalCount())
		})
	}

	_, _, err := searchProjects(withProjects(&fakeProjects{}), interfaces.ProjectSearchParam{},
		&flowv1.ProjectSort{Field: flowv1.ProjectSortField(99)}, nil)
	assert.Equal(t, codes.InvalidArgument, status.Code(err))
}

func TestFindProject(t *testing.T) {
	p := project.New().NewID().Workspace(accountsid.NewWorkspaceID()).Alias("roads").MustBuild()
	ctx := withProjects(&fakeProjects{project: p})
	byAlias := func(ws, alias string) *flowv1.ProjectRef {
		return &flowv1.ProjectRef{Ref: &flowv1.ProjectRef_Alias{Alias: &flowv1.ProjectAlias{WorkspaceId: ws, Alias: alias}}}
	}
	byID := func(s string) *flowv1.ProjectRef {
		return &flowv1.ProjectRef{Ref: &flowv1.ProjectRef_Id{Id: s}}
	}

	got, err := findProject(ctx, byID(p.ID().String()))
	require.NoError(t, err)
	assert.Equal(t, p.ID(), got.ID())

	got, err = findProject(ctx, byAlias(p.Workspace().String(), "roads"))
	require.NoError(t, err)
	assert.Equal(t, p.ID(), got.ID())

	tests := []struct {
		ref  *flowv1.ProjectRef
		name string
		want codes.Code
	}{
		{nil, "no project", codes.InvalidArgument},
		{&flowv1.ProjectRef{}, "empty ref", codes.InvalidArgument},
		{byID("not-an-id"), "bad id", codes.InvalidArgument},
		{byAlias("not-an-id", "roads"), "bad workspace", codes.InvalidArgument},
		{byAlias(p.Workspace().String(), ""), "empty alias", codes.InvalidArgument},
		{byID(id.NewProjectID().String()), "unknown id", codes.NotFound},
		{byAlias(p.Workspace().String(), "other"), "unknown alias", codes.NotFound},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := findProject(ctx, tt.ref)
			assert.Equal(t, tt.want, status.Code(err))
		})
	}
}
