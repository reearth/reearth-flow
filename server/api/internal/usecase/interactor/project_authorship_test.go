package interactor

import (
	"context"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/samber/lo"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestProject_Create_RecordsCreator(t *testing.T) {
	uc, ws, ctx, u := setupProjectMetadata(t, allowAll())

	got, err := uc.Create(ctx, interfaces.CreateProjectParam{WorkspaceID: ws.ID()})
	require.NoError(t, err)
	assert.Equal(t, lo.ToPtr(u.ID()), got.CreatedBy())
	assert.Equal(t, lo.ToPtr(u.ID()), got.UpdatedBy(), "the creator is the first editor")
}

func TestProject_Update_RecordsEditor(t *testing.T) {
	uc, ws, ctx, creator := setupProjectMetadata(t, allowAll())
	p, err := uc.Create(ctx, interfaces.CreateProjectParam{WorkspaceID: ws.ID()})
	require.NoError(t, err)

	editor := accountsuser.New().NewID().Name("fuga").Email("fuga@bb.cc").MustBuild()
	editorCtx := adapter.AttachUser(context.Background(), editor)

	got, err := uc.Update(editorCtx, interfaces.UpdateProjectParam{ID: p.ID(), Name: lo.ToPtr("renamed")})
	require.NoError(t, err)
	assert.Equal(t, lo.ToPtr(editor.ID()), got.UpdatedBy())
	assert.Equal(t, lo.ToPtr(creator.ID()), got.CreatedBy(), "an edit never changes the creator")

	got, err = uc.Update(ctx, interfaces.UpdateProjectParam{ID: p.ID()})
	require.NoError(t, err)
	assert.Equal(t, lo.ToPtr(editor.ID()), got.UpdatedBy(), "an update that sets nothing is not an edit")
}

func TestProject_Update_WithoutUserKeepsEditor(t *testing.T) {
	uc, ws, ctx, u := setupProjectMetadata(t, allowAll())
	p, err := uc.Create(ctx, interfaces.CreateProjectParam{WorkspaceID: ws.ID()})
	require.NoError(t, err)

	got, err := uc.Update(context.Background(), interfaces.UpdateProjectParam{ID: p.ID(), Name: lo.ToPtr("renamed")})
	require.NoError(t, err)
	assert.Equal(t, "renamed", got.Name())
	assert.Equal(t, lo.ToPtr(u.ID()), got.UpdatedBy())
}

func TestProject_FindByWorkspace_CreatedBy(t *testing.T) {
	uc, ws, ctx, u := setupProjectMetadata(t, allowAll())
	mine, err := uc.Create(ctx, interfaces.CreateProjectParam{WorkspaceID: ws.ID()})
	require.NoError(t, err)

	other := accountsuser.New().NewID().Name("fuga").Email("fuga@bb.cc").MustBuild()
	_, err = uc.Create(adapter.AttachUser(context.Background(), other), interfaces.CreateProjectParam{WorkspaceID: ws.ID()})
	require.NoError(t, err)

	// Saved before creators were recorded: matched by no creator.
	legacy := project.New().NewID().Workspace(ws.ID()).UpdatedAt(time.Now()).MustBuild()
	require.NoError(t, uc.projectRepo.Save(ctx, legacy))

	got, _, err := uc.FindByWorkspace(ctx, ws.ID(), nil, nil, nil, accountsid.UserIDList{u.ID()})
	require.NoError(t, err)
	require.Len(t, got, 1)
	assert.Equal(t, mine.ID(), got[0].ID())

	got, _, err = uc.FindByWorkspace(ctx, ws.ID(), nil, nil, nil, accountsid.UserIDList{u.ID(), other.ID()})
	require.NoError(t, err)
	assert.Len(t, got, 2, "several creators match any of them")

	got, _, err = uc.FindByWorkspace(ctx, ws.ID(), nil, nil, nil, nil)
	require.NoError(t, err)
	assert.Len(t, got, 3, "no creator filter lists every project")

	got, _, err = uc.FindByWorkspace(ctx, ws.ID(), nil, nil, nil, accountsid.UserIDList{})
	require.NoError(t, err)
	assert.Len(t, got, 3, "an empty creator filter lists every project")

	got, _, err = uc.FindByWorkspace(ctx, ws.ID(), nil, nil, nil, accountsid.UserIDList{accountsid.NewUserID()})
	require.NoError(t, err)
	assert.Empty(t, got)
}

func TestProjectAccess_ShareRecordsEditor(t *testing.T) {
	mem := memory.New()
	i := &ProjectAccess{
		projectRepo:       mem.Project,
		projectAccessRepo: mem.ProjectAccess,
		transaction:       mem.Transaction,
		config:            ContainerConfig{Host: "https://example.com", SharedPath: "shared"},
		permissionChecker: allowAll(),
	}

	creator := accountsid.NewUserID()
	prj := project.New().NewID().Workspace(accountsid.NewWorkspaceID()).
		CreatedBy(&creator).UpdatedBy(&creator).UpdatedAt(time.Now()).MustBuild()
	require.NoError(t, mem.Project.Save(context.Background(), prj))

	sharer := accountsuser.New().NewID().Name("fuga").Email("fuga@bb.cc").MustBuild()
	ctx := adapter.AttachUser(context.Background(), sharer)

	_, err := i.Share(ctx, prj.ID())
	require.NoError(t, err)
	got, err := mem.Project.FindByID(ctx, prj.ID())
	require.NoError(t, err)
	assert.Equal(t, lo.ToPtr(sharer.ID()), got.UpdatedBy())
	assert.Equal(t, &creator, got.CreatedBy())

	unsharer := accountsuser.New().NewID().Name("piyo").Email("piyo@bb.cc").MustBuild()
	require.NoError(t, i.Unshare(adapter.AttachUser(context.Background(), unsharer), prj.ID()))
	got, err = mem.Project.FindByID(ctx, prj.ID())
	require.NoError(t, err)
	assert.Equal(t, lo.ToPtr(unsharer.ID()), got.UpdatedBy())
}
