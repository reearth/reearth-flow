package project

import (
	"fmt"
	"strings"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestValidateAlias(t *testing.T) {
	pid := NewID()
	other := NewID()

	tests := []struct {
		err   error
		name  string
		alias string
	}{
		{name: "letters digits dash underscore", alias: "My_proj-01"},
		{name: "shortest", alias: "abcde"},
		{name: "longest", alias: strings.Repeat("a", 32)},
		{name: "own default alias", alias: DefaultAlias(pid)},
		{name: "too short", alias: "abcd", err: ErrInvalidAlias},
		{name: "too long", alias: strings.Repeat("a", 33), err: ErrInvalidAlias},
		{name: "space", alias: "my project", err: ErrInvalidAlias},
		{name: "slash", alias: "my/project", err: ErrInvalidAlias},
		{name: "non-ascii", alias: "プロジェクト名前", err: ErrInvalidAlias},
		{name: "another project's default alias", alias: DefaultAlias(other), err: ErrReservedAlias},
		{name: "reserved prefix", alias: "p-mine", err: ErrReservedAlias},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			assert.Equal(t, tt.err, ValidateAlias(tt.alias, pid))
		})
	}

	// Without a project (checking availability before create), no default
	// alias is the caller's own, including one built from the zero ID.
	var none ID
	assert.Equal(t, ErrReservedAlias, ValidateAlias("p-00000000000000000000000000", none))
	assert.Equal(t, ErrReservedAlias, ValidateAlias(DefaultAlias(pid), none))
	assert.Equal(t, ErrInvalidAlias, ValidateAlias(DefaultAlias(none), none))
}

func TestDefaultAlias_IsValidForItsProject(t *testing.T) {
	pid := NewID()
	assert.NoError(t, ValidateAlias(DefaultAlias(pid), pid))
}

func TestNormalizeTopics(t *testing.T) {
	got, err := NormalizeTopics([]string{" gis ", "", "plateau", "gis", "  "})
	require.NoError(t, err)
	assert.Equal(t, []string{"gis", "plateau"}, got)

	got, err = NormalizeTopics(nil)
	require.NoError(t, err)
	assert.Empty(t, got)

	_, err = NormalizeTopics([]string{strings.Repeat("あ", MaxTopicLength)})
	assert.NoError(t, err, "length is counted in characters, not bytes")

	_, err = NormalizeTopics([]string{strings.Repeat("a", MaxTopicLength+1)})
	assert.Equal(t, ErrTopicTooLong, err)

	many := make([]string, MaxTopics+1)
	for i := range many {
		many[i] = fmt.Sprintf("topic-%d", i)
	}
	_, err = NormalizeTopics(many)
	assert.Equal(t, ErrTooManyTopics, err)

	_, err = NormalizeTopics(append(many[:MaxTopics], many[0]))
	assert.NoError(t, err, "repeats do not count toward the limit")
}

func TestProject_SetAlias(t *testing.T) {
	p := New().NewID().Workspace(accountsid.NewWorkspaceID()).MustBuild()
	def := p.Alias()

	assert.Equal(t, ErrInvalidAlias, p.SetAlias("bad"))
	assert.Equal(t, def, p.Alias(), "a rejected alias leaves the project unchanged")

	require.NoError(t, p.SetAlias("my-project"))
	assert.Equal(t, "my-project", p.Alias())

	require.NoError(t, p.SetAlias(def), "a project can return to its default alias")
}

func TestProject_SetVisibility(t *testing.T) {
	p := New().NewID().MustBuild()
	assert.Equal(t, VisibilityPrivate, p.Visibility())

	require.NoError(t, p.SetVisibility(VisibilityPublic))
	assert.Equal(t, VisibilityPublic, p.Visibility())

	assert.Equal(t, ErrInvalidVisibility, p.SetVisibility("unlisted"))
	assert.Equal(t, VisibilityPublic, p.Visibility())
}

func TestProject_SetTopics(t *testing.T) {
	p := New().NewID().MustBuild()
	require.NoError(t, p.SetTopics([]string{"a", "a", " b "}))
	assert.Equal(t, []string{"a", "b"}, p.Topics())

	assert.Equal(t, ErrTopicTooLong, p.SetTopics([]string{strings.Repeat("x", MaxTopicLength+1)}))
	assert.Equal(t, []string{"a", "b"}, p.Topics())
}

func TestProject_Stars(t *testing.T) {
	u1 := accountsid.NewUserID()
	u2 := accountsid.NewUserID()
	p := New().NewID().MustBuild()
	updatedAt := p.UpdatedAt()

	p.Star(u1)
	p.Star(u1)
	p.Star(u2)
	assert.Equal(t, int64(2), p.StarCount(), "starring twice counts once")
	assert.True(t, p.IsStarredBy(u1))

	p.Unstar(u1)
	p.Unstar(u1)
	assert.Equal(t, int64(1), p.StarCount())
	assert.False(t, p.IsStarredBy(u1))
	assert.True(t, p.IsStarredBy(u2))

	assert.Equal(t, updatedAt, p.UpdatedAt(), "stars are not edits")
}

func TestBuilder_KeepsStoredMetadata(t *testing.T) {
	pid := NewID()
	topics := []string{"gis"}
	starredBy := []string{accountsid.NewUserID().String()}

	p := New().ID(pid).
		Alias("custom-alias").
		Visibility(VisibilityPublic).
		Readme("# Readme").
		License("MIT").
		Topics(topics).
		StarredBy(starredBy).
		MustBuild()

	assert.Equal(t, "custom-alias", p.Alias())
	assert.Equal(t, VisibilityPublic, p.Visibility())
	assert.Equal(t, "# Readme", p.Readme())
	assert.Equal(t, "MIT", p.License())
	assert.Equal(t, topics, p.Topics())
	assert.Equal(t, int64(1), p.StarCount())

	topics[0] = "changed"
	p.Topics()[0] = "changed"
	assert.Equal(t, []string{"gis"}, p.Topics(), "the project owns its slices")
}
