package trigger

import (
	"crypto/rand"
	"crypto/sha256"
	"crypto/subtle"
	"encoding/hex"
	"time"
)

// AuthTokenPrefix starts every generated token, so a caller can tell one apart
// in their configuration.
const AuthTokenPrefix = "flow_trigger_"

const (
	authTokenRandomBytes = 32
	authTokenHintLength  = 4
)

// HashAuthToken is what is stored in place of a token. A generated token is
// random enough that a plain SHA-256 cannot be reversed; the migration that
// hashed the tokens saved before generation computes the same value in SQL.
func HashAuthToken(token string) string {
	sum := sha256.Sum256([]byte(token))
	return hex.EncodeToString(sum[:])
}

// GenerateAuthToken gives the trigger a new random token, which replaces any
// earlier one, and returns it. Only its hash and last characters are kept, so
// it cannot be read back later.
func (t *Trigger) GenerateAuthToken() string {
	random := make([]byte, authTokenRandomBytes)
	_, _ = rand.Read(random) // crypto/rand.Read never returns an error
	token := AuthTokenPrefix + hex.EncodeToString(random)

	hash := HashAuthToken(token)
	hint := token[len(token)-authTokenHintLength:]
	t.authTokenHash = &hash
	t.authTokenHint = &hint
	t.updatedAt = time.Now()
	return token
}

// VerifyAuthToken reports whether token is the trigger's token. The hashes
// are compared in constant time, so the time taken does not show how much of
// a guess was right.
func (t *Trigger) VerifyAuthToken(token string) bool {
	if t.authTokenHash == nil || token == "" {
		return false
	}
	return subtle.ConstantTimeCompare([]byte(HashAuthToken(token)), []byte(*t.authTokenHash)) == 1
}

func (t *Trigger) ClearAuthToken() {
	t.authTokenHash = nil
	t.authTokenHint = nil
	t.updatedAt = time.Now()
}

func (t *Trigger) AuthTokenHash() *string {
	return t.authTokenHash
}

// AuthTokenHint is the token's last characters, for telling tokens apart. It
// is nil for a token saved before tokens were generated, since those may be
// short enough for the hint to give them away.
func (t *Trigger) AuthTokenHint() *string {
	return t.authTokenHint
}
