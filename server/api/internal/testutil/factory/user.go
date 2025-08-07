package factory

import (
	faker "github.com/go-faker/faker/v4"
	"github.com/reearth/reearth-flow/api/pkg/user"
	"golang.org/x/text/language"
)

type UserOption func(*user.Builder)

func NewUser(opts ...UserOption) *user.User {
	metadata := user.NewMetadata().
		Description(faker.Sentence()).
		Lang(language.English).
		Theme("light").
		Website(faker.URL()).
		PhotoURL(faker.URL()).
		MustBuild()

	p := user.New().
		ID(user.NewID()).
		Name(faker.Name()).
		Alias(faker.Username()).
		Email(faker.Email()).
		Metadata(metadata)
	for _, opt := range opts {
		opt(p)
	}
	return p.MustBuild()
}
