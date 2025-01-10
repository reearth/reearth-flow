package app

import (
	"context"
	"errors"

	"github.com/labstack/echo/v4"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountusecase/accountinteractor"
	"github.com/reearth/reearthx/account/accountusecase/accountinterfaces"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/rerror"
)

const (
	debugUserHeader = "X-Reearth-Debug-User"
)

// attachUserMiddleware attaches user to the context
func attachUserMiddleware(cfg *ServerConfig) echo.MiddlewareFunc {
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		multiUser := accountinteractor.NewMultiUser(cfg.AccountRepos, cfg.AccountGateways, cfg.Config.SignupSecret, cfg.Config.Host_Web, cfg.AccountRepos.Users)
		return func(c echo.Context) error {
			ctx := c.Request().Context()
			u, err := getUser(ctx, c, multiUser, cfg)
			if err != nil {
				return err
			}

			if u != nil {
				ctx = attachUserToContext(ctx, u, cfg)
				if err != nil {
					return err
				}
			}

			c.SetRequest(c.Request().WithContext(ctx))
			return next(c)
		}
	}
}

func getUser(ctx context.Context, c echo.Context, multiUser accountinterfaces.User, cfg *ServerConfig) (*user.User, error) {
	au := adapter.GetAuthInfo(ctx)
	if au != nil {
		u, err := multiUser.FetchBySub(ctx, au.Sub)
		if err != nil && !errors.Is(err, rerror.ErrNotFound) {
			return nil, err
		}
		if u != nil {
			if err := addAuth0SubToUser(ctx, u, user.AuthFrom(au.Sub), cfg); err != nil {
				return nil, err
			}
			return u, nil
		}
	}

	if cfg.Debug {
		userID := c.Request().Header.Get(debugUserHeader)
		if userID != "" {
			if uID, err := accountdomain.UserIDFrom(userID); err == nil {
				users, err := multiUser.FetchByID(ctx, user.IDList{uID})
				if err == nil && len(users) == 1 {
					return users[0], nil
				}
			}
		}
	}

	return nil, nil
}

func attachUserToContext(ctx context.Context, u *user.User, _ *ServerConfig) context.Context {
	ctx = adapter.AttachUser(ctx, u)
	log.Debugfc(ctx, "auth: user: id=%s name=%s email=%s", u.ID(), u.Name(), u.Email())
	return ctx
}

func addAuth0SubToUser(ctx context.Context, u *user.User, a user.Auth, cfg *ServerConfig) error {
	if u.AddAuth(a) {
		err := cfg.Repos.User.Save(ctx, u)
		if err != nil {
			return err
		}
	}
	return nil
}
