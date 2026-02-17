package factory

import (
	faker "github.com/go-faker/faker/v4"
	"github.com/reearth/reearth-accounts/server/pkg/user"
	"golang.org/x/text/language"
)

type UserOption func(*user.Builder)

func NewUser(opts ...UserOption) *user.User {
	metadata := user.NewMetadata()
	metadata.SetDescription(faker.Sentence())
	metadata.SetLang(language.English)
	metadata.SetTheme(user.ThemeDefault)
	metadata.SetWebsite(faker.URL())
	metadata.SetPhotoURL(faker.URL())

	b := user.New().
		NewID().
		Name(faker.Name()).
		Alias(faker.Username()).
		Email(faker.Email()).
		Metadata(metadata)
	for _, opt := range opts {
		opt(b)
	}
	return b.MustBuild()
}
