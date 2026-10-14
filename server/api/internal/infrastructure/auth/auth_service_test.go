package auth

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// fakeProjects records what the handler asked for and returns canned verdicts.
type fakeProjects struct {
	err error

	gotProject string
	gotJWT     string
	gotShare   string

	read, edit, shared bool
}

func (f *fakeProjects) UserAccess(ctx context.Context, projectID string) (bool, bool, error) {
	f.gotProject = projectID
	f.gotJWT = adapter.JWT(ctx)
	return f.read, f.edit, f.err
}

func (f *fakeProjects) SharedAccess(_ context.Context, shareToken, projectID string) (bool, error) {
	f.gotProject = projectID
	f.gotShare = shareToken
	return f.shared, f.err
}

func newTestHandler(projects ProjectAccessChecker) *AuthHandler {
	h := NewAuthHandler(nil, projects)
	h.validate = func(_ context.Context, token string) error {
		if token != "good" {
			return assert.AnError
		}
		return nil
	}
	return h
}

func verify(t *testing.T, h *AuthHandler, req TokenVerifyRequest) (int, TokenVerifyResponse) {
	t.Helper()
	body, err := json.Marshal(req)
	require.NoError(t, err)
	rec := httptest.NewRecorder()
	h.VerifyAPI(rec, httptest.NewRequest(http.MethodPost, "/auth/verify", bytes.NewReader(body)))
	var resp TokenVerifyResponse
	if rec.Code == http.StatusOK || rec.Code == http.StatusForbidden {
		require.NoError(t, json.NewDecoder(rec.Body).Decode(&resp))
	}
	return rec.Code, resp
}

func TestVerifyRoom_MemberWithEditGetsReadWrite(t *testing.T) {
	p := &fakeProjects{read: true, edit: true}
	code, resp := verify(t, newTestHandler(p), TokenVerifyRequest{Token: "Bearer good", Room: "proj-1:main"})
	assert.Equal(t, http.StatusOK, code)
	assert.True(t, resp.Authorized)
	assert.False(t, resp.ReadOnly)
	assert.Equal(t, "proj-1", p.gotProject, "the project is the room's prefix before the first colon")
	assert.Equal(t, "good", p.gotJWT, "the accounts calls must carry the caller's JWT")
}

func TestVerifyRoom_ReaderGetsReadOnly(t *testing.T) {
	p := &fakeProjects{read: true}
	code, resp := verify(t, newTestHandler(p), TokenVerifyRequest{Token: "good", Room: "proj-1:other-graph"})
	assert.Equal(t, http.StatusOK, code)
	assert.True(t, resp.Authorized)
	assert.True(t, resp.ReadOnly)
	assert.Equal(t, "proj-1", p.gotProject)
}

func TestVerifyRoom_ValidTokenWithoutProjectAccessDenied(t *testing.T) {
	code, resp := verify(t, newTestHandler(&fakeProjects{}), TokenVerifyRequest{Token: "good", Room: "victim:main"})
	assert.Equal(t, http.StatusForbidden, code)
	assert.False(t, resp.Authorized)
}

func TestVerifyRoom_InvalidTokenDenied(t *testing.T) {
	p := &fakeProjects{read: true, edit: true}
	code, _ := verify(t, newTestHandler(p), TokenVerifyRequest{Token: "forged", Room: "proj-1:main"})
	assert.Equal(t, http.StatusUnauthorized, code)
	assert.Empty(t, p.gotProject, "project access must not be checked for an invalid token")
}

func TestVerifyRoom_ShareTokenGetsReadOnly(t *testing.T) {
	p := &fakeProjects{shared: true}
	code, resp := verify(t, newTestHandler(p), TokenVerifyRequest{ShareToken: "s1", Room: "proj-1:main"})
	assert.Equal(t, http.StatusOK, code)
	assert.True(t, resp.Authorized)
	assert.True(t, resp.ReadOnly, "shared-link viewers must never get write access")
	assert.Equal(t, "s1", p.gotShare)
	assert.Equal(t, "proj-1", p.gotProject)
}

func TestVerifyRoom_BadShareTokenDenied(t *testing.T) {
	code, resp := verify(t, newTestHandler(&fakeProjects{}), TokenVerifyRequest{ShareToken: "s1", Room: "proj-1:main"})
	assert.Equal(t, http.StatusForbidden, code)
	assert.False(t, resp.Authorized)
}

func TestVerifyRoom_NoCredentialDenied(t *testing.T) {
	code, resp := verify(t, newTestHandler(&fakeProjects{read: true, edit: true, shared: true}), TokenVerifyRequest{Room: "proj-1:main"})
	assert.Equal(t, http.StatusForbidden, code)
	assert.False(t, resp.Authorized)
}

func TestVerifyRoom_AccessErrorDenied(t *testing.T) {
	code, _ := verify(t, newTestHandler(&fakeProjects{read: true, err: assert.AnError}), TokenVerifyRequest{Token: "good", Room: "proj-1:main"})
	assert.Equal(t, http.StatusInternalServerError, code)
}

func TestVerifyRoom_NoCheckerFailsClosed(t *testing.T) {
	code, resp := verify(t, newTestHandler(nil), TokenVerifyRequest{Token: "good", Room: "proj-1:main"})
	assert.Equal(t, http.StatusForbidden, code)
	assert.False(t, resp.Authorized)
}

// TestVerifyLegacy_TokenOnly keeps the Rust server's request shape working: no
// room means the token-only check it has always had.
func TestVerifyLegacy_TokenOnly(t *testing.T) {
	code, resp := verify(t, newTestHandler(nil), TokenVerifyRequest{Token: "good"})
	assert.Equal(t, http.StatusOK, code)
	assert.True(t, resp.Authorized)

	code, _ = verify(t, newTestHandler(nil), TokenVerifyRequest{Token: "forged"})
	assert.Equal(t, http.StatusUnauthorized, code)
}
