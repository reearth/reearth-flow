package interactor

import (
	"context"
	"errors"
	"strings"
	"testing"

	gqlworkspace "github.com/reearth/reearth-accounts/server/pkg/gqlclient/workspace"
	accountsworkspace "github.com/reearth/reearth-accounts/server/pkg/workspace"
	"github.com/reearth/reearthx/util"
	"github.com/samber/lo"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// recordingWorkspaceGQLRepo captures the aliases Create sends and can reject the first n the way accounts rejects a taken alias.
type recordingWorkspaceGQLRepo struct {
	gqlworkspace.WorkspaceRepo
	sent        []string
	rejectFirst int
}

func (r *recordingWorkspaceGQLRepo) CreateWorkspace(_ context.Context, in gqlworkspace.CreateWorkspaceInput) (*accountsworkspace.Workspace, error) {
	r.sent = append(r.sent, in.Alias)
	if len(r.sent) <= r.rejectFirst {
		return nil, errors.New("workspace alias already exists")
	}
	return nil, nil
}

func TestDeriveAlias(t *testing.T) {
	tests := []struct {
		name string
		in   string
		want string
	}{
		{"spaces become hyphens", "My Workspace", "my-workspace"},
		{"trimmed and collapsed", "  Spaced   Out  ", "spaced-out"},
		{"underscores are not allowed", "PLATEAU_Tokyo", "plateau-tokyo"},
		{"consecutive hyphens collapse", "a--b", "a-baa"},
		{"leading and trailing hyphens go", "-lead-", "leada"},
		{"short names are padded to the minimum", "ML", "mlaaa"},
		{"symbols are dropped", "R&D", "r-daa"},
		{"long names are capped", strings.Repeat("ab", 40), strings.Repeat("ab", 15)},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			assert.Equal(t, tc.want, deriveAlias(tc.in))
		})
	}
}

// The alias namespace is global, so a shared padded value would let the first workspace lock out every other one.
func TestDeriveAliasFallsBackToAUniqueValue(t *testing.T) {
	first := deriveAlias("東京都データ基盤")
	second := deriveAlias("大阪府データ基盤")

	assert.True(t, strings.HasPrefix(first, "w-"), "got %q", first)
	assert.NotEqual(t, first, second)
}

// Asserted against the rule accounts actually applies, not a copy of it.
func TestDeriveAliasAlwaysPassesTheAccountsRule(t *testing.T) {
	names := []string{
		"My Workspace", "東京都データ基盤", "ML", "a", "", "   ", "-", "---",
		"R&D", "PLATEAU_Tokyo", "🌏🌏🌏", "Tokyo/Osaka", strings.Repeat("ab", 40),
		"café", "a--b", "-lead-", "123", ".", "test",
	}

	for _, name := range names {
		alias := deriveAlias(name)
		assert.True(t, util.IsSafePathName(alias), "name %q derived invalid alias %q", name, alias)
	}
}

func TestAliasCandidate(t *testing.T) {
	assert.Equal(t, "my-workspace-2", aliasCandidate("my-workspace", 2))
	assert.True(t, util.IsSafePathName(aliasCandidate(strings.Repeat("ab", 15), 10)))
	assert.LessOrEqual(t, len(aliasCandidate(strings.Repeat("ab", 15), 10)), aliasHardMaxLen)
}

func TestWorkspaceCreateSendsTheGivenAlias(t *testing.T) {
	repo := &recordingWorkspaceGQLRepo{}
	i := NewWorkspace(repo, &recordingChecker{allow: true})

	_, err := i.Create(context.Background(), "My Workspace", lo.ToPtr("chosen-alias"))

	require.NoError(t, err)
	assert.Equal(t, []string{"chosen-alias"}, repo.sent)
}

func TestWorkspaceCreateDerivesAnOmittedAlias(t *testing.T) {
	repo := &recordingWorkspaceGQLRepo{}
	i := NewWorkspace(repo, &recordingChecker{allow: true})

	_, err := i.Create(context.Background(), "My Workspace", nil)

	require.NoError(t, err)
	assert.Equal(t, []string{"my-workspace"}, repo.sent)
}

// An empty alias is a value the caller sent, not an absent one: accounts rejects it, and that rejection is the answer.
func TestWorkspaceCreateDoesNotDeriveAnEmptyAlias(t *testing.T) {
	repo := &recordingWorkspaceGQLRepo{}
	i := NewWorkspace(repo, &recordingChecker{allow: true})

	_, err := i.Create(context.Background(), "My Workspace", lo.ToPtr(""))

	require.NoError(t, err)
	assert.Equal(t, []string{""}, repo.sent, "an explicit empty alias must reach accounts, not be derived away")
}

func TestWorkspaceCreateRetriesADerivedAliasThatIsTaken(t *testing.T) {
	repo := &recordingWorkspaceGQLRepo{rejectFirst: 2}
	i := NewWorkspace(repo, &recordingChecker{allow: true})

	_, err := i.Create(context.Background(), "My Workspace", nil)

	require.NoError(t, err)
	assert.Equal(t, []string{"my-workspace", "my-workspace-2", "my-workspace-3"}, repo.sent)
}

// An alias the caller chose is theirs: report the duplicate rather than create under a different handle.
func TestWorkspaceCreateDoesNotRetryAChosenAlias(t *testing.T) {
	repo := &recordingWorkspaceGQLRepo{rejectFirst: 1}
	i := NewWorkspace(repo, &recordingChecker{allow: true})

	_, err := i.Create(context.Background(), "My Workspace", lo.ToPtr("chosen-alias"))

	assert.Error(t, err)
	assert.Equal(t, []string{"chosen-alias"}, repo.sent)
}

func TestWorkspaceCreateGivesUpAfterTheRetryBudget(t *testing.T) {
	repo := &recordingWorkspaceGQLRepo{rejectFirst: 99}
	i := NewWorkspace(repo, &recordingChecker{allow: true})

	_, err := i.Create(context.Background(), "My Workspace", nil)

	assert.Error(t, err)
	assert.Len(t, repo.sent, aliasCreateAttempts)
}
