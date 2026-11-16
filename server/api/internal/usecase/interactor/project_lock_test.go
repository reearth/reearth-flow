package interactor

import (
	"context"
	"errors"
	"fmt"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/samber/lo"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// setupProjectLock returns a Project interactor whose websocket client records
// lock signals, and one saved project.
func setupProjectLock(t *testing.T) (*Project, *mockWebsocketClient, context.Context, interfaces.UpdateProjectParam) {
	t.Helper()
	uc, ws, ctx, _ := setupProjectMetadata(t, allowAll())
	client := &mockWebsocketClient{}
	uc.websocket = client
	uc.lockSignalBackoff = 0
	p, err := uc.Create(ctx, interfaces.CreateProjectParam{WorkspaceID: ws.ID()})
	require.NoError(t, err)
	return uc, client, ctx, interfaces.UpdateProjectParam{ID: p.ID()}
}

func TestProject_Update_SignalsALockChange(t *testing.T) {
	t.Parallel()

	uc, client, ctx, param := setupProjectLock(t)

	param.IsLocked = lo.ToPtr(true)
	got, err := uc.Update(ctx, param)
	require.NoError(t, err)
	require.Len(t, client.lockCalls, 1)
	assert.Equal(t, lockSignal{docID: got.ID().String(), locked: true, changedAt: got.UpdatedAt()}, client.lockCalls[0])

	param.IsLocked = lo.ToPtr(false)
	got, err = uc.Update(ctx, param)
	require.NoError(t, err)
	require.Len(t, client.lockCalls, 2)
	assert.Equal(t, lockSignal{docID: got.ID().String(), locked: false, changedAt: got.UpdatedAt()}, client.lockCalls[1])
}

func TestProject_Update_DoesNotSignalWithoutALockChange(t *testing.T) {
	t.Parallel()

	uc, client, ctx, param := setupProjectLock(t)

	param.IsLocked = lo.ToPtr(false) // already unlocked
	_, err := uc.Update(ctx, param)
	require.NoError(t, err)

	param.IsLocked = nil
	param.Name = lo.ToPtr("renamed")
	_, err = uc.Update(ctx, param)
	require.NoError(t, err)

	assert.Empty(t, client.lockCalls)
}

func TestProject_Update_RetriesALockSignal(t *testing.T) {
	t.Parallel()

	uc, client, ctx, param := setupProjectLock(t)
	client.lockErrs = []error{errors.New("unavailable")}

	param.IsLocked = lo.ToPtr(true)
	_, err := uc.Update(ctx, param)
	require.NoError(t, err)
	assert.Len(t, client.lockCalls, 2, "a failed signal is tried again")
}

func TestProject_Update_BoundsEachLockSignalAttempt(t *testing.T) {
	t.Parallel()

	uc, client, ctx, param := setupProjectLock(t)
	client.lockErrs = []error{errors.New("timed out")}

	param.IsLocked = lo.ToPtr(true)
	_, err := uc.Update(ctx, param)
	require.NoError(t, err)
	require.Len(t, client.lockBudgets, 2, "every attempt has a deadline")
	for _, budget := range client.lockBudgets {
		assert.LessOrEqual(t, budget, lockSignalAttemptTimeout)
	}
}

func TestProject_Update_DoesNotRetryARefusedLockSignal(t *testing.T) {
	t.Parallel()

	uc, client, ctx, param := setupProjectLock(t)
	client.lockErrs = []error{fmt.Errorf("%w: 401", interfaces.ErrWebsocketRequestRejected)}

	param.IsLocked = lo.ToPtr(true)
	_, err := uc.Update(ctx, param)
	require.NoError(t, err)
	assert.Len(t, client.lockCalls, 1, "a refusal cannot succeed on a second try")
}

func TestProject_Update_SavesTheLockWhenTheSignalFails(t *testing.T) {
	t.Parallel()

	uc, client, ctx, param := setupProjectLock(t)
	fail := errors.New("unavailable")
	client.lockErrs = []error{fail, fail, fail, fail}

	param.IsLocked = lo.ToPtr(true)
	got, err := uc.Update(ctx, param)
	require.NoError(t, err, "the lock is saved; new connections are checked against it")
	assert.True(t, got.IsLocked())
	assert.Len(t, client.lockCalls, lockSignalAttempts)

	saved, err := uc.projectRepo.FindByID(ctx, param.ID)
	require.NoError(t, err)
	assert.True(t, saved.IsLocked())
}
