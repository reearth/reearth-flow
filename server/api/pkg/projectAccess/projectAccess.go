package projectAccess

import (
	"crypto/rand"
	"encoding/base64"
	"errors"
	"fmt"
)

var (
	ErrAlreadyPublic   = errors.New("project is already public")
	ErrAlreadyPrivate  = errors.New("project is already private")
	ErrNotPublic       = errors.New("project is not public")
	ErrEmptyBaseURL    = errors.New("baseURL is empty")
	ErrEmptySharedPath = errors.New("sharedPath is empty")
	ErrEmptyToken      = errors.New("token is empty")
)

type ProjectAccess struct {
	token    string
	id       ID
	project  ProjectID
	isPublic bool
}

func (pa *ProjectAccess) ID() ID {
	return pa.id
}

func (pa *ProjectAccess) Project() ProjectID {
	return pa.project
}

func (pa *ProjectAccess) IsPublic() bool {
	return pa.isPublic
}

func (pa *ProjectAccess) Token() string {
	return pa.token
}

func (pa *ProjectAccess) SetIsPublic(isPublic bool) {
	pa.isPublic = isPublic
}

func (pa *ProjectAccess) SetToken(token string) {
	pa.token = token
}

func (pa *ProjectAccess) MakePublic() error {
	if pa.isPublic {
		return ErrAlreadyPublic
	}

	pa.isPublic = true

	token, err := generateToken()
	if err != nil {
		return err
	}
	pa.token = token

	return nil
}

func (pa *ProjectAccess) MakePrivate() error {
	if !pa.isPublic {
		return ErrAlreadyPrivate
	}

	pa.isPublic = false
	pa.token = ""

	return nil
}

func (pa *ProjectAccess) SharingURL(baseURL string, sharedUrl string) (url string, err error) {
	if !pa.IsPublic() {
		return "", ErrNotPublic
	}
	if baseURL == "" {
		return "", ErrEmptyBaseURL
	}
	if sharedUrl == "" {
		return "", ErrEmptySharedPath
	}
	if pa.token == "" {
		return "", ErrEmptyToken
	}
	return fmt.Sprintf("%s/%s/%s", baseURL, sharedUrl, pa.token), nil
}

func generateToken() (string, error) {
	b := make([]byte, 32)
	if _, err := rand.Read(b); err != nil {
		return "", fmt.Errorf("failed to generate random bytes: %w", err)
	}

	token := base64.URLEncoding.EncodeToString(b)

	return fmt.Sprintf("shr_%s", token), nil
}
