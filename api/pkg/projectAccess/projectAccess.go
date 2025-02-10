package projectAccess

import (
	"crypto/rand"
	"encoding/base64"
	"errors"
	"fmt"
	"time"
)

var (
	ErrAlreadyPublic  = errors.New("project is already public")
	ErrAlreadyPrivate = errors.New("project is already private")
	ErrNotPublic      = errors.New("project is not public")
	ErrEmptyBaseURL   = errors.New("baseURL is empty")
	ErrEmptyToken     = errors.New("token is empty")
)

type ProjectAccess struct {
	id        ID
	project   ProjectID
	isPublic  bool
	token     string
	updatedAt time.Time
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

func (pa *ProjectAccess) CreatedAt() time.Time {
	return pa.id.Timestamp()
}

func (pa *ProjectAccess) UpdatedAt() time.Time {
	return pa.updatedAt
}

func (pa *ProjectAccess) SetIsPublic(isPublic bool) {
	pa.isPublic = isPublic
}

func (pa *ProjectAccess) SetToken(token string) {
	pa.token = token
}

func (pa *ProjectAccess) SharingURL(baseURL string) (url string, err error) {
	if !pa.IsPublic() {
		return "", ErrNotPublic
	}
	if baseURL == "" {
		return "", ErrEmptyBaseURL
	}
	if pa.token == "" {
		return "", ErrEmptyToken
	}
	return fmt.Sprintf("%s/shared/%s", baseURL, pa.token), nil
}

func (pa *ProjectAccess) MakePublic() error {
	if pa.isPublic {
		return ErrAlreadyPrivate
	}

	pa.isPublic = true

	token, err := generateToken()
	if err != nil {
		return err
	}
	pa.token = token

	pa.updatedAt = time.Now()

	return nil
}

func (pa *ProjectAccess) MakePrivate() error {
	if !pa.isPublic {
		return ErrAlreadyPublic
	}

	pa.isPublic = false
	pa.token = ""

	pa.updatedAt = time.Now()

	return nil
}

func generateToken() (string, error) {
	b := make([]byte, 32)
	if _, err := rand.Read(b); err != nil {
		return "", fmt.Errorf("failed to generate random bytes: %w", err)
	}

	token := base64.URLEncoding.EncodeToString(b)

	return fmt.Sprintf("shr_%s", token), nil
}
