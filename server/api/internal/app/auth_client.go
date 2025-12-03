package app

import (
	"context"
	"errors"

	"github.com/labstack/echo/v4"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/reearth/reearthx/account/accountusecase"
	"github.com/reearth/reearthx/account/accountusecase/accountinteractor"
	"github.com/reearth/reearthx/account/accountusecase/accountinterfaces"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/rerror"
)

// TODO: Remove this file once the migration is complete.
const (
	debugUserHeader = "X-Reearth-Debug-User"
)

// attachOpMiddleware attaches the operator and user to the context
func attachOpMiddleware(cfg *ServerConfig) echo.MiddlewareFunc {
	return func(next echo.HandlerFunc) echo.HandlerFunc {
		multiUser := accountinteractor.NewMultiUser(cfg.AccountRepos, cfg.AccountGateways, cfg.Config.SignupSecret, cfg.Config.HostWeb, cfg.AccountRepos.Users)
		return func(c echo.Context) error {
			ctx := c.Request().Context()
			u, err := getUser(ctx, c, multiUser, cfg)
			if err != nil {
				return err
			}

			if u != nil {
				ctx = attachUserToContext(ctx, u, cfg)
				op, err := generateOperator(ctx, cfg, u)
				if err != nil {
					return err
				}
				ctx = adapter.AttachOperator(ctx, op)
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
	ctx = adapter.AttachReearthxUser(ctx, u)
	log.Debugfc(ctx, "auth: user: id=%s name=%s email=%s", u.ID(), u.Name(), u.Email())
	return ctx
}

func generateOperator(ctx context.Context, cfg *ServerConfig, u *user.User) (*usecase.Operator, error) {
	if u == nil {
		return nil, nil
	}

	uid := u.ID()
	workspaces, err := cfg.Repos.Workspace.FindByUser(ctx, uid)
	if err != nil {
		return nil, err
	}

	readableWorkspaces := workspaces.FilterByUserRole(uid, workspace.RoleReader).IDs()
	writableWorkspaces := workspaces.FilterByUserRole(uid, workspace.RoleWriter).IDs()
	maintainingWorkspaces := workspaces.FilterByUserRole(uid, workspace.RoleMaintainer).IDs()
	owningWorkspaces := workspaces.FilterByUserRole(uid, workspace.RoleOwner).IDs()

	return &usecase.Operator{
		AcOperator: &accountusecase.Operator{
			User:                   &uid,
			ReadableWorkspaces:     readableWorkspaces,
			WritableWorkspaces:     writableWorkspaces,
			MaintainableWorkspaces: maintainingWorkspaces,
			OwningWorkspaces:       owningWorkspaces,
		},
	}, nil
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
