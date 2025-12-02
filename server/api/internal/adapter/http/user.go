package http

import (
	"context"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"golang.org/x/text/language"
)

type UserController struct {
	usecase interfaces.User
}

func NewUserController(usecase interfaces.User) *UserController {
	return &UserController{
		usecase: usecase,
	}
}

type PasswordResetInput struct {
	Email    string `json:"email"`
	Token    string `json:"token"`
	Password string `json:"password"`
}

type SignupInput struct {
	Sub         *string                 `json:"sub"`
	Secret      *string                 `json:"secret"`
	UserID      *accountsid.UserID      `json:"userId"`
	WorkspaceID *accountsid.WorkspaceID `json:"workspaceId"`
	TeamID      *accountsid.WorkspaceID `json:"teamId"` // TeamID is an alias of WorkspaceID
	Name        string                  `json:"name"`
	Username    string                  `json:"username"` // ysername is an alias of Name
	Email       string                  `json:"email"`
	Password    string                  `json:"password"`
	Theme       *accountsuser.Theme     `json:"theme"`
	Lang        *language.Tag           `json:"lang"`
}

type CreateVerificationInput struct {
	Email string `json:"email"`
}

type VerifyUserOutput struct {
	UserID   string `json:"userId"`
	Verified bool   `json:"verified"`
}

type SignupOutput struct {
	ID    string `json:"id"`
	Name  string `json:"name"`
	Email string `json:"email"`
}

func (c *UserController) Signup(ctx context.Context, input SignupInput) (SignupOutput, error) {
	if input.Name == "" && input.Username != "" {
		input.Name = input.Username
	}
	if input.WorkspaceID == nil && input.TeamID != nil {
		input.WorkspaceID = input.TeamID
	}

	if input.Sub != nil && *input.Sub != "" && input.Email != "" && input.Name != "" {
		u, err := c.usecase.SignupOIDC(ctx, interfaces.SignupOIDCParam{
			Name:   &input.Name,
			Email:  &input.Email,
			Sub:    input.Sub,
			Secret: input.Secret,
		})
		if err != nil {
			return SignupOutput{}, err
		}

		return SignupOutput{
			ID:    u.ID().String(),
			Name:  u.Name(),
			Email: u.Email(),
		}, nil
	}

	u, err := c.usecase.Signup(ctx, interfaces.SignupParam{
		Name:        input.Name,
		Email:       input.Email,
		Password:    input.Password,
		Secret:      input.Secret,
		UserID:      input.UserID,
		WorkspaceID: input.WorkspaceID,
		Lang:        input.Lang,
		Theme:       input.Theme,
	})
	if err != nil {
		return SignupOutput{}, err
	}

	return SignupOutput{
		ID:    u.ID().String(),
		Name:  u.Name(),
		Email: u.Email(),
	}, nil
}

func (c *UserController) CreateVerification(ctx context.Context, input CreateVerificationInput) error {
	return c.usecase.CreateVerification(ctx, input.Email)
}

func (c *UserController) VerifyUser(ctx context.Context, code string) (VerifyUserOutput, error) {
	u, err := c.usecase.VerifyUser(ctx, code)
	if err != nil {
		return VerifyUserOutput{}, err
	}
	return VerifyUserOutput{
		UserID:   u.ID().String(),
		Verified: true,
	}, nil
}

func (c *UserController) StartPasswordReset(ctx context.Context, input PasswordResetInput) error {
	return c.usecase.StartPasswordReset(ctx, input.Email)
}

func (c *UserController) PasswordReset(ctx context.Context, input PasswordResetInput) error {
	return c.usecase.PasswordReset(ctx, input.Password, input.Token)
}
