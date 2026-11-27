package adapter

import (
	"context"

	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearthx/appx"
)

type (
	userKey             struct{}
	jwtTokenKey         struct{}
	gqlOperationNameKey struct{}
)

type ContextKey string

const (
	ContextAuthInfo ContextKey = "authinfo"
	contextUsecases ContextKey = "usecases"
)

type AuthInfo struct {
	EmailVerified *bool
	Token         string
	Sub           string
	Iss           string
	Name          string
	Email         string
}

func AttachUser(ctx context.Context, u *accountsuser.User) context.Context {
	return context.WithValue(ctx, userKey{}, u)
}

func AttachAuthInfo(ctx context.Context, authInfo *appx.AuthInfo) context.Context {
	return context.WithValue(ctx, ContextAuthInfo, *authInfo)
}

func AttachUsecases(ctx context.Context, u *interfaces.Container) context.Context {
	ctx = context.WithValue(ctx, contextUsecases, u)
	return ctx
}

func AttachJWT(ctx context.Context, token string) context.Context {
	return context.WithValue(ctx, jwtTokenKey{}, token)
}

func AttachGQLOperationName(ctx context.Context, op string) context.Context {
	return context.WithValue(ctx, gqlOperationNameKey{}, op)
}

func User(ctx context.Context) *accountsuser.User {
	u, _ := ctx.Value(userKey{}).(*accountsuser.User)
	return u
}

func Usecases(ctx context.Context) *interfaces.Container {
	return ctx.Value(contextUsecases).(*interfaces.Container)
}

func JWT(ctx context.Context) string {
	t, _ := ctx.Value(jwtTokenKey{}).(string)
	return t
}

func GQLOperationName(ctx context.Context) string {
	t, _ := ctx.Value(gqlOperationNameKey{}).(string)
	return t
}

// TODO: Remove this function once the migration to accounts server is complete.
func TempAuthInfo(ctx context.Context) *appx.AuthInfo {
	if authInfo, ok := ctx.Value("authinfo").(appx.AuthInfo); ok {
		return &authInfo
	}
	return nil
}
