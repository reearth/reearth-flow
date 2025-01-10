package adapter

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/appx"
	"golang.org/x/text/language"
)

type ContextKey string

const (
	contextUser     ContextKey = "user"
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

func AttachUsecases(ctx context.Context, u *interfaces.Container) context.Context {
	ctx = context.WithValue(ctx, contextUsecases, u)
	return ctx
}

func User(ctx context.Context) *user.User {
	if v := ctx.Value(contextUser); v != nil {
		if u, ok := v.(*user.User); ok {
			return u
		}
	}
	return nil
}

func Lang(ctx context.Context, lang *language.Tag) string {
	if lang != nil && !lang.IsRoot() {
		return lang.String()
	}

	u := User(ctx)
	if u == nil {
		return defaultLang.String()
	}

	l := u.Lang()
	if l.IsRoot() {
		return defaultLang.String()
	}

	return l.String()
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
