package gqlmodel

import (
	"testing"

	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/stretchr/testify/assert"
	"golang.org/x/text/language"
)

func newTestUser(t *testing.T, photoURL string) *accountsuser.User {
	t.Helper()

	return accountsuser.New().
		NewID().
		Name("Test User").
		Email("test@example.com").
		Workspace(accountsuser.NewWorkspaceID()).
		Metadata(accountsuser.MetadataFrom(photoURL, "", "", language.English, accountsuser.ThemeDefault)).
		MustBuild()
}

func TestToMe(t *testing.T) {
	t.Parallel()

	t.Run("exposes the photo URL", func(t *testing.T) {
		t.Parallel()

		me := ToMe(newTestUser(t, "https://example.com/photo.png"))

		assert.NotNil(t, me.Metadata)
		assert.Equal(t, "https://example.com/photo.png", *me.Metadata.PhotoURL)
	})

	t.Run("leaves the photo URL nil when unset", func(t *testing.T) {
		t.Parallel()

		me := ToMe(newTestUser(t, ""))

		assert.NotNil(t, me.Metadata)
		assert.Nil(t, me.Metadata.PhotoURL)
	})

	t.Run("returns nil for a nil user", func(t *testing.T) {
		t.Parallel()

		assert.Nil(t, ToMe(nil))
	})
}
