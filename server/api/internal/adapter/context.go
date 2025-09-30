package adapter

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/user"
	reearthxuser "github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/appx"
	"golang.org/x/text/language"
)

type userKey struct{}
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

// TODO: After migration, remove this function
func AttachReearthxUser(ctx context.Context, u *reearthxuser.User) context.Context {
	return context.WithValue(ctx, contextUser, u)
}

func AttachUser(ctx context.Context, u *user.User) context.Context {
	return context.WithValue(ctx, userKey{}, u)
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

// TODO: After migration, remove this function
func ReearthxUser(ctx context.Context) *reearthxuser.User {
	if v := ctx.Value(contextUser); v != nil {
		if u, ok := v.(*reearthxuser.User); ok {
			return u
		}
	}
	return nil
}

func User(ctx context.Context) *user.User {
	u, _ := ctx.Value(userKey{}).(*user.User)
	return u
}

func Lang(ctx context.Context, lang *language.Tag) string {
	if lang != nil && !lang.IsRoot() {
		return lang.String()
	}

	u := ReearthxUser(ctx)
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
