package adapter

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	pkguser "github.com/reearth/reearth-flow/api/pkg/user"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/appx"
	"golang.org/x/text/language"
)

type flowUserKey struct{}
type jwtTokenKey struct{}
type gqlOperationNameKey struct{}

type ContextKey string

const (
	contextUser     ContextKey = "user"
	contextOperator ContextKey = "operator"
	ContextAuthInfo ContextKey = "authinfo"
	contextUsecases ContextKey = "usecases"
)

var defaultLang = language.English

type AuthInfo struct {
	Token         string
	Sub           string
	Iss           string
	Name          string
	Email         string
	EmailVerified *bool
}

func AttachUser(ctx context.Context, u *user.User) context.Context {
	return context.WithValue(ctx, contextUser, u)
}

// TODO: Keep using AttachUser during the migration period.
// After migration, unify it so that AttachUser returns a FlowUser (from flow/pkg).
func AttachFlowUser(ctx context.Context, u *pkguser.User) context.Context {
	return context.WithValue(ctx, flowUserKey{}, u)
}

func AttachOperator(ctx context.Context, o *usecase.Operator) context.Context {
	return context.WithValue(ctx, contextOperator, o)
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

func User(ctx context.Context) *user.User {
	if v := ctx.Value(contextUser); v != nil {
		if u, ok := v.(*user.User); ok {
			return u
		}
	}
	return nil
}

// TODO: Keep using User during the migration period.
// After migration, unify it so that User returns a FlowUser (from flow/pkg).
func FlowUser(ctx context.Context) *pkguser.User {
	u, _ := ctx.Value(flowUserKey{}).(*pkguser.User)
	return u
}

func Lang(ctx context.Context, lang *language.Tag) string {
	if lang != nil && !lang.IsRoot() {
		return lang.String()
	}

	u := User(ctx)
	if u == nil {
		return defaultLang.String()
	}

	if u.Metadata() != nil {
		l := u.Metadata().Lang()
		if !l.IsRoot() {
			return l.String()
		}
	}
	return defaultLang.String()
}

func Operator(ctx context.Context) *usecase.Operator {
	if v := ctx.Value(contextOperator); v != nil {
		if v2, ok := v.(*usecase.Operator); ok {
			return v2
		}
	}
	return nil
}

func GetAuthInfo(ctx context.Context) *appx.AuthInfo {
	if v := ctx.Value(ContextAuthInfo); v != nil {
		if v2, ok := v.(appx.AuthInfo); ok {
			return &v2
		}
	}
	return nil
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
