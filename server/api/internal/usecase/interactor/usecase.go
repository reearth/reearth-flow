package interactor

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/usecasex"
)

const retry = 2

type uc struct {
	tx                 bool
	readableWorkspaces id.WorkspaceIDList
	writableWorkspaces id.WorkspaceIDList
}

func Usecase() *uc {
	return &uc{}
}

func (u *uc) WithReadableWorkspaces(ids ...id.WorkspaceID) *uc {
	u.readableWorkspaces = id.WorkspaceIDList(ids).Clone()
	return u
}

func (u *uc) WithWritableWorkspaces(ids ...id.WorkspaceID) *uc {
	u.writableWorkspaces = id.WorkspaceIDList(ids).Clone()
	return u
}

func (u *uc) Transaction() *uc {
	u.tx = true
	return u
}

func Run0(ctx context.Context, r *repo.Container, e *uc, f func(ctx context.Context) error) (err error) {
	_, _, _, err = Run3(
		ctx, r, e,
		func(ctx context.Context) (_, _, _ any, err error) {
			err = f(ctx)
			return
		})
	return
}

func Run1[A any](ctx context.Context, r *repo.Container, e *uc, f func(ctx context.Context) (A, error)) (a A, err error) {
	a, _, _, err = Run3(
		ctx, r, e,
		func(ctx context.Context) (a A, _, _ any, err error) {
			a, err = f(ctx)
			return
		})
	return
}

func Run2[A, B any](ctx context.Context, r *repo.Container, e *uc, f func(ctx context.Context) (A, B, error)) (a A, b B, err error) {
	a, b, _, err = Run3(
		ctx, r, e,
		func(ctx context.Context) (a A, b B, _ any, err error) {
			a, b, err = f(ctx)
			return
		})
	return
}

func Run3[A, B, C any](ctx context.Context, r *repo.Container, e *uc, f func(ctx context.Context) (A, B, C, error)) (a A, b B, c C, err error) {
	var t usecasex.Transaction
	if e.tx && r.Transaction != nil {
		t = r.Transaction
	}

	err = usecasex.DoTransaction(ctx, t, retry, func(ctx context.Context) error {
		a, b, c, err = f(ctx)
		return err
	})
	return
}
