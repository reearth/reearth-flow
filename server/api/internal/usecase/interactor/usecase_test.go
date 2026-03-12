package interactor

import (
	"context"
	"errors"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearthx/usecasex"
	"github.com/stretchr/testify/assert"
)

func TestUc(t *testing.T) {
	workspaces := accountsid.WorkspaceIDList{accountsid.NewWorkspaceID(), accountsid.NewWorkspaceID(), accountsid.NewWorkspaceID()}
	assert.Equal(t, &uc{}, Usecase())
	assert.Equal(t, &uc{readableWorkspaces: workspaces}, (&uc{}).WithReadableWorkspaces(workspaces...))
	assert.Equal(t, &uc{writableWorkspaces: workspaces}, (&uc{}).WithWritableWorkspaces(workspaces...))
	assert.Equal(t, &uc{tx: true}, (&uc{}).Transaction())
}

func TestRun(t *testing.T) {
	ctx := context.Background()
	err := errors.New("test")
	a, b, c := &struct{}{}, &struct{}{}, &struct{}{}

	// regular1: without tx
	tr := &usecasex.NopTransaction{}
	r := &repo.Container{Transaction: tr}
	gota, gotb, gotc, goterr := Run3(
		ctx, r,
		Usecase(),
		func(ctx context.Context) (any, any, any, error) {
			return a, b, c, nil
		},
	)
	assert.Same(t, a, gota)
	assert.Same(t, b, gotb)
	assert.Same(t, c, gotc)
	assert.Nil(t, goterr)
	assert.False(t, tr.IsCommitted()) // not IsCommitted

	// regular2: with tx
	tr = &usecasex.NopTransaction{}
	r.Transaction = tr
	_ = Run0(
		ctx, r,
		Usecase().Transaction(),
		func(ctx context.Context) error {
			return nil
		},
	)
	assert.True(t, tr.IsCommitted())

	// iregular1: the usecase returns an error
	tr = &usecasex.NopTransaction{}
	r.Transaction = tr
	goterr = Run0(
		ctx, r,
		Usecase().Transaction(),
		func(ctx context.Context) error {
			return err
		},
	)
	assert.Same(t, err, goterr)
	assert.False(t, tr.IsCommitted())

	// iregular2: tx.Begin returns an error
	tr = &usecasex.NopTransaction{}
	r.Transaction = tr
	tr.BeginError = err
	tr.CommitError = nil
	goterr = Run0(
		ctx, r,
		Usecase().Transaction(),
		func(ctx context.Context) error {
			return nil
		},
	)
	assert.Same(t, err, goterr)
	assert.False(t, tr.IsCommitted())

	// iregular3: tx.End returns an error
	tr = &usecasex.NopTransaction{}
	r.Transaction = tr
	tr.BeginError = nil
	tr.CommitError = err
	goterr = Run0(
		ctx, r,
		Usecase().Transaction(),
		func(ctx context.Context) error {
			return nil
		},
	)
	assert.Same(t, err, goterr)
	assert.True(t, tr.IsCommitted())
}
