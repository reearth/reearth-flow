package projectAccess

import (
	"encoding/base64"
	"strings"
	"testing"

	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/stretchr/testify/assert"
)

func TestProjectAccess_ID(t *testing.T) {
	expectedID := NewID()
	pa := &ProjectAccess{id: expectedID}
	assert.Equal(t, expectedID, pa.ID())
}

func TestProjectAccess_Project(t *testing.T) {
	expectedID := NewProjectID()
	pa := &ProjectAccess{project: expectedID}
	assert.Equal(t, expectedID, pa.Project())
}

func TestProjectAccess_IsPublic(t *testing.T) {
	pa := &ProjectAccess{isPublic: true}
	assert.Equal(t, true, pa.IsPublic())

	pa = &ProjectAccess{isPublic: false}
	assert.Equal(t, false, pa.IsPublic())
}

func TestProjectAccess_Token(t *testing.T) {
	expectedToken := "token"
	pa := &ProjectAccess{token: expectedToken}
	assert.Equal(t, expectedToken, pa.Token())
}

func TestProjectAccess_UserRoles(t *testing.T) {
	expectedUserRoles := NewUserRoleList()
	pa := &ProjectAccess{userRoles: expectedUserRoles}
	assert.Equal(t, expectedUserRoles, pa.UserRoles())
}

func TestProjectAccess_SetIsPublic(t *testing.T) {
	pa := &ProjectAccess{}
	pa.SetIsPublic(true)
	assert.Equal(t, true, pa.isPublic)

	pa.SetIsPublic(false)
	assert.Equal(t, false, pa.isPublic)
}

func TestProjectAccess_SetToken(t *testing.T) {
	pa := &ProjectAccess{}
	pa.SetToken("token")
	assert.Equal(t, "token", pa.token)
}

func TestProjectAccess_MakePublic(t *testing.T) {
	tests := []struct {
		name       string
		pa         *ProjectAccess
		wantErr    error
		wantPublic bool
		wantToken  bool
	}{
		{
			name: "success: private to public",
			pa: &ProjectAccess{
				isPublic: false,
				token:    "",
			},
			wantErr:    nil,
			wantPublic: true,
			wantToken:  true,
		},
		{
			name: "failure: already public",
			pa: &ProjectAccess{
				isPublic: true,
				token:    "existing_token",
			},
			wantErr:    ErrAlreadyPublic,
			wantPublic: true,
			wantToken:  true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.pa.MakePublic()
			assert.ErrorIs(t, err, tt.wantErr)

			assert.Equal(t, tt.wantPublic, tt.pa.isPublic)
			if tt.wantToken {
				assert.NotEmpty(t, tt.pa.token)
			} else {
				assert.Empty(t, tt.pa.token)
			}
		})
	}
}

func TestProjectAccess_MakePrivate(t *testing.T) {
	tests := []struct {
		name       string
		pa         *ProjectAccess
		wantErr    error
		wantPublic bool
		wantToken  string
	}{
		{
			name: "success: public to private",
			pa: &ProjectAccess{
				isPublic: true,
				token:    "some_token",
			},
			wantErr:    nil,
			wantPublic: false,
			wantToken:  "",
		},
		{
			name: "failure: already private",
			pa: &ProjectAccess{
				isPublic: false,
				token:    "",
			},
			wantErr:    ErrAlreadyPrivate,
			wantPublic: false,
			wantToken:  "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.pa.MakePrivate()
			assert.ErrorIs(t, err, tt.wantErr)
			assert.Equal(t, tt.wantPublic, tt.pa.isPublic)
			assert.Equal(t, tt.wantToken, tt.pa.token)
		})
	}
}

func TestProjectAccess_SharingURL(t *testing.T) {
	tests := []struct {
		name       string
		pa         *ProjectAccess
		baseURL    string
		sharedPath string
		wantURL    string
		wantErr    error
	}{
		{
			name: "success: valid URL generation",
			pa: &ProjectAccess{
				isPublic: true,
				token:    "test_token",
			},
			baseURL:    "https://example.com",
			sharedPath: "shared",
			wantURL:    "https://example.com/shared/test_token",
			wantErr:    nil,
		},
		{
			name: "failure: project not public",
			pa: &ProjectAccess{
				isPublic: false,
				token:    "test_token",
			},
			baseURL:    "https://example.com",
			sharedPath: "shared",
			wantURL:    "",
			wantErr:    ErrNotPublic,
		},
		{
			name: "failure: empty baseURL",
			pa: &ProjectAccess{
				isPublic: true,
				token:    "test_token",
			},
			baseURL:    "",
			sharedPath: "shared",
			wantURL:    "",
			wantErr:    ErrEmptyBaseURL,
		},
		{
			name: "failure: empty sharedPath",
			pa: &ProjectAccess{
				isPublic: true,
				token:    "test_token",
			},
			baseURL:    "https://example.com",
			sharedPath: "",
			wantURL:    "",
			wantErr:    ErrEmptySharedPath,
		},
		{
			name: "failure: empty token",
			pa: &ProjectAccess{
				isPublic: true,
				token:    "",
			},
			baseURL:    "https://example.com",
			sharedPath: "shared",
			wantURL:    "",
			wantErr:    ErrEmptyToken,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			url, err := tt.pa.SharingURL(tt.baseURL, tt.sharedPath)
			assert.ErrorIs(t, err, tt.wantErr)
			assert.Equal(t, tt.wantURL, url)
		})
	}
}

func Test_generateToken(t *testing.T) {
	// Generate a token multiple times to ensure each one is different.
	tokens := make(map[string]bool)
	for i := 0; i < 100; i++ {
		token, err := generateToken()
		assert.NoError(t, err)
		assert.True(t, strings.HasPrefix(token, "shr_"))

		assert.Greater(t, len(token), 32)

		assert.False(t, tokens[token], "duplicate token generated")
		tokens[token] = true

		tokenWithoutPrefix := strings.TrimPrefix(token, "shr_")
		_, err = base64.URLEncoding.DecodeString(tokenWithoutPrefix)
		assert.NoError(t, err, "token should be valid base64")
	}
}

func TestProjectAccess_AddUserRole(t *testing.T) {
	userID := user.NewID()
	pa := &ProjectAccess{
		userRoles: NewUserRoleList(),
	}

	// Add a user (success case)
	err := pa.AddUserRole(userID, workspace.RoleOwner)
	assert.NoError(t, err)
	assert.Len(t, pa.userRoles, 1)
	assert.Equal(t, userID, pa.userRoles[0].UserID())
	assert.Equal(t, workspace.RoleOwner, pa.userRoles[0].Role())

	// Add duplicate user (failure case)
	err = pa.AddUserRole(userID, workspace.RoleReader)
	assert.Error(t, err)
	assert.Equal(t, ErrUserRoleExists, err)
	assert.Len(t, pa.userRoles, 1)
}

func TestProjectAccess_EditUserRole(t *testing.T) {
	userID1 := user.NewID()
	userID2 := user.NewID()
	pa := &ProjectAccess{
		userRoles: NewUserRoleList(),
	}

	_ = pa.userRoles.Add(userID1, workspace.RoleOwner)

	// Update the permission of an existing user to a different value (successful case)
	err := pa.EditUserRole(userID1, workspace.RoleReader)
	assert.NoError(t, err)
	assert.Equal(t, workspace.RoleReader, pa.userRoles[0].Role())

	// Update the permission of an existing user to the same value (failure case - no change)
	err = pa.EditUserRole(userID1, workspace.RoleReader)
	assert.Error(t, err)
	assert.Equal(t, ErrNoRoleChange, err)

	// Update the permission of a non-existent user (failure case)
	err = pa.EditUserRole(userID2, workspace.RoleWriter)
	assert.Error(t, err)
	assert.Equal(t, ErrUserRoleNotFound, err)
}

func TestProjectAccess_RemoveUserRole(t *testing.T) {
	userID1 := user.NewID()
	userID2 := user.NewID()
	pa := &ProjectAccess{
		userRoles: NewUserRoleList(),
	}

	_ = pa.userRoles.Add(userID1, workspace.RoleOwner)

	// Removing an existing user (successful case)
	err := pa.RemoveUserRole(userID1)
	assert.NoError(t, err)
	assert.Empty(t, pa.userRoles)

	// Removing a non-existent user (failure case)
	err = pa.RemoveUserRole(userID2)
	assert.Error(t, err)
	assert.Equal(t, ErrUserRoleNotFound, err)
}
