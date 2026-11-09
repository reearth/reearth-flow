package mongodoc

import (
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestProjectDocument_Authorship(t *testing.T) {
	creator, editor := accountsid.NewUserID(), accountsid.NewUserID()
	p := project.New().NewID().Workspace(accountsid.NewWorkspaceID()).
		CreatedBy(&creator).UpdatedBy(&editor).UpdatedAt(time.Now()).MustBuild()

	doc, _ := NewProject(p)
	require.NotNil(t, doc.CreatedBy)
	assert.Equal(t, creator.String(), *doc.CreatedBy)

	got, err := doc.Model()
	require.NoError(t, err)
	assert.Equal(t, &creator, got.CreatedBy())
	assert.Equal(t, &editor, got.UpdatedBy())
}

func TestProjectDocument_AuthorshipAbsent(t *testing.T) {
	// A document saved before authorship was recorded has neither field.
	p := project.New().NewID().Workspace(accountsid.NewWorkspaceID()).UpdatedAt(time.Now()).MustBuild()

	doc, _ := NewProject(p)
	assert.Nil(t, doc.CreatedBy)
	assert.Nil(t, doc.UpdatedBy)

	got, err := doc.Model()
	require.NoError(t, err)
	assert.Nil(t, got.CreatedBy())
	assert.Nil(t, got.UpdatedBy())
}
