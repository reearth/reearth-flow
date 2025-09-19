package user

import (
	"golang.org/x/text/language"
)

type SignupAttrs struct {
	ID          *ID
	WorkspaceID *WorkspaceID
	Name        string
	Email       string
	Password    string
	Secret      *string
	Lang        *language.Tag
	Theme       *Theme
	MockAuth    bool
}
