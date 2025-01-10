package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountusecase"
)

type ContextKey string

const (
	contextLoaders     ContextKey = "loaders"
	contextDataloaders ContextKey = "dataloaders"
)

func AttachUsecases(ctx context.Context, u *interfaces.Container, enableDataLoaders bool) context.Context {
	loaders := NewLoaders(u)
	dataloaders := loaders.DataLoadersWith(ctx, enableDataLoaders)

	ctx = adapter.AttachUsecases(ctx, u)
	ctx = context.WithValue(ctx, contextLoaders, loaders)
	ctx = context.WithValue(ctx, contextDataloaders, dataloaders)

	return ctx
}

func getUser(ctx context.Context) *user.User {
	return adapter.User(ctx)
}

// Temporarily returns nil because the operator is defined in reearthx interface.
// This implementation is a temporary workaround and will be removed in the future.
// TODO: After fixing the reearthx module (removing operator implementation and modifying interfaces),
// this function and all its usages should be deleted.
func getAcOperator() *accountusecase.Operator {
	return nil
}

func usecases(ctx context.Context) *interfaces.Container {
	return adapter.Usecases(ctx)
}

func loaders(ctx context.Context) *Loaders {
	return ctx.Value(contextLoaders).(*Loaders)
}

func dataloaders(ctx context.Context) *DataLoaders {
	return ctx.Value(contextDataloaders).(*DataLoaders)
}

// func intToInt64(i *int) *int64 {
// 	if i == nil {
// 		return nil
// 	}
// 	return lo.ToPtr(int64(*i))
// }
