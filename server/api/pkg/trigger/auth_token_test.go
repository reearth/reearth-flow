package trigger

import (
	"encoding/hex"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestHashAuthToken(t *testing.T) {
	// The migration hashes existing tokens in SQL, so the format is pinned: the
	// lowercase hex SHA-256 of the token's bytes.
	assert.Equal(t, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad", HashAuthToken("abc"))
}

func TestTrigger_GenerateAuthToken(t *testing.T) {
	tr := &Trigger{}
	token := tr.GenerateAuthToken()

	require.True(t, strings.HasPrefix(token, AuthTokenPrefix), token)
	random, err := hex.DecodeString(strings.TrimPrefix(token, AuthTokenPrefix))
	require.NoError(t, err)
	assert.Len(t, random, 32)

	require.NotNil(t, tr.AuthTokenHash())
	assert.Equal(t, HashAuthToken(token), *tr.AuthTokenHash())
	assert.NotContains(t, *tr.AuthTokenHash(), strings.TrimPrefix(token, AuthTokenPrefix))
	require.NotNil(t, tr.AuthTokenHint())
	assert.Equal(t, token[len(token)-4:], *tr.AuthTokenHint())
	assert.True(t, tr.VerifyAuthToken(token))

	assert.NotEqual(t, token, (&Trigger{}).GenerateAuthToken())
}

func TestTrigger_GenerateAuthToken_ReplacesOldToken(t *testing.T) {
	tr := &Trigger{}
	old := tr.GenerateAuthToken()
	token := tr.GenerateAuthToken()

	assert.False(t, tr.VerifyAuthToken(old))
	assert.True(t, tr.VerifyAuthToken(token))
}

func TestTrigger_VerifyAuthToken(t *testing.T) {
	legacy := New().AuthTokenHash(HashAuthToken("abc")).t
	empty := New().AuthTokenHash(HashAuthToken("")).t

	tests := []struct {
		name    string
		trigger *Trigger
		token   string
		want    bool
	}{
		{"a token saved before tokens were generated", legacy, "abc", true},
		{"a wrong token", legacy, "abd", false},
		{"no token", legacy, "", false},
		{"an empty token saved as a hash", empty, "", false},
		{"a trigger without a token", &Trigger{}, "abc", false},
		{"a trigger without a token, no token", &Trigger{}, "", false},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			assert.Equal(t, tt.want, tt.trigger.VerifyAuthToken(tt.token))
		})
	}
}

func TestTrigger_ClearAuthToken(t *testing.T) {
	tr := &Trigger{}
	token := tr.GenerateAuthToken()
	tr.ClearAuthToken()

	assert.Nil(t, tr.AuthTokenHash())
	assert.Nil(t, tr.AuthTokenHint())
	assert.False(t, tr.VerifyAuthToken(token))
}
