package user

import (
	"golang.org/x/text/language"
)

type SignupOIDCAttrs struct {
	UserID      *ID
	Lang        *language.Tag
	WorkspaceID *WorkspaceID
	Secret      *string
}
