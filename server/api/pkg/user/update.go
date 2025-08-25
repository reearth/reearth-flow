package user

import "golang.org/x/text/language"

type UpdateAttrs struct {
	Name                 *string
	Email                *string
	Lang                 *language.Tag
	Password             *string
	PasswordConfirmation *string
}
