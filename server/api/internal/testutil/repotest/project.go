// Package repotest holds behaviour tests that every repository backend must
// pass, so Mongo, Postgres and the in-memory fakes are held to one contract.
package repotest

import (
	"context"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/samber/lo"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// ProjectSearch checks repo.Project.Search: filters, sorting, paging and the
// readable-workspace filter.
func ProjectSearch(t *testing.T, r repo.Project) {
	t.Helper()
	ctx := context.Background()
	a, b, other := accountsid.NewWorkspaceID(), accountsid.NewWorkspaceID(), accountsid.NewWorkspaceID()
	base := time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)

	type seed struct {
		name     string
		vis      project.Visibility
		topics   []string
		ws       accountsid.WorkspaceID
		stars    int
		archived bool
	}
	seeds := []seed{
		{"alpha", project.VisibilityPublic, []string{"gis", "plateau"}, a, 2, false},
		{"beta", project.VisibilityPrivate, []string{"gis"}, a, 0, false},
		{"gamma 50%", project.VisibilityPublic, nil, a, 1, true},
		{"delta", project.VisibilityPublic, []string{"plateau"}, b, 3, false},
		{"epsilon", project.VisibilityPublic, []string{"gis"}, other, 5, false},
	}
	for i, s := range seeds {
		p := project.New().NewID().Workspace(s.ws).Name(s.name).Visibility(s.vis).
			Topics(s.topics).IsArchived(s.archived).
			UpdatedAt(base.Add(time.Duration(i) * time.Hour)).MustBuild()
		require.NoError(t, r.Save(ctx, p))
		for range s.stars {
			require.NoError(t, r.SetStar(ctx, p.ID(), accountsid.NewUserID(), true))
		}
	}

	// Searches across workspaces see only a and b, so other databases' or
	// other tests' projects never leak in.
	scoped := r.Filtered(repo.WorkspaceFilter{Readable: accountsid.WorkspaceIDList{a, b}})
	search := func(p interfaces.ProjectSearchParam) ([]string, int64) {
		t.Helper()
		if p.Page == 0 {
			p.Page, p.PageSize = 1, 10
		}
		got, info, err := scoped.Search(ctx, p)
		require.NoError(t, err)
		return lo.Map(got, func(p *project.Project, _ int) string { return p.Name() }), info.TotalCount
	}
	public := lo.ToPtr(project.VisibilityPublic)

	got, total := search(interfaces.ProjectSearchParam{Workspace: &a})
	assert.Equal(t, []string{"beta", "alpha"}, got, "newest first, archived left out")
	assert.Equal(t, int64(2), total)

	got, _ = search(interfaces.ProjectSearchParam{Workspace: &a, IncludeArchived: true, SortBy: interfaces.ProjectSortName, SortAscending: true})
	assert.Equal(t, []string{"alpha", "beta", "gamma 50%"}, got)

	got, _ = search(interfaces.ProjectSearchParam{Visibility: public, SortBy: interfaces.ProjectSortName, SortAscending: true})
	assert.Equal(t, []string{"alpha", "delta"}, got, "every readable workspace")

	got, _ = search(interfaces.ProjectSearchParam{Workspace: &a, Visibility: lo.ToPtr(project.VisibilityPrivate)})
	assert.Equal(t, []string{"beta"}, got)

	got, _ = search(interfaces.ProjectSearchParam{Topics: []string{"gis", "plateau"}})
	assert.Equal(t, []string{"alpha"}, got, "every topic must match")

	got, _ = search(interfaces.ProjectSearchParam{Visibility: public, SortBy: interfaces.ProjectSortStarCount})
	assert.Equal(t, []string{"delta", "alpha"}, got, "most stars first")

	got, _ = search(interfaces.ProjectSearchParam{Visibility: public, SortBy: interfaces.ProjectSortStarCount, SortAscending: true})
	assert.Equal(t, []string{"alpha", "delta"}, got)

	got, _ = search(interfaces.ProjectSearchParam{Keyword: "ALP"})
	assert.Equal(t, []string{"alpha"}, got, "keywords ignore case")

	got, _ = search(interfaces.ProjectSearchParam{Keyword: "%", IncludeArchived: true})
	assert.Equal(t, []string{"gamma 50%"}, got, "a keyword matches literally")

	got, _ = search(interfaces.ProjectSearchParam{Keyword: "a_p", IncludeArchived: true})
	assert.Empty(t, got, "_ is not a wildcard")

	got, total = search(interfaces.ProjectSearchParam{SortBy: interfaces.ProjectSortName, SortAscending: true, Page: 2, PageSize: 2})
	assert.Equal(t, []string{"delta"}, got)
	assert.Equal(t, int64(3), total)

	got, total = search(interfaces.ProjectSearchParam{Page: 5, PageSize: 2})
	assert.Empty(t, got, "past the last page")
	assert.Equal(t, int64(3), total)
}
