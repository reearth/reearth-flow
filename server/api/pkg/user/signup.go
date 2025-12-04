package user

import (
	"golang.org/x/text/language"
)

type SignupAttrs struct {
	ID          *ID
	WorkspaceID *WorkspaceID
	Secret      *string
	Lang        *language.Tag
	Theme       *Theme
	Name        string
	Email       string
	Password    string
	MockAuth    bool
}
