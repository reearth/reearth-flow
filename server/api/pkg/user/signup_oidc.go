package user

import (
	"golang.org/x/text/language"
)

type SignupOIDCAttrs struct {
	UserID      *ID
	Name        *string
	Email       *string
	Sub         *string
	Lang        *language.Tag
	WorkspaceID *WorkspaceID
	Secret      *string
}
