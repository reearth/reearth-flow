package interactor

import (
	"fmt"
	"math/rand/v2"
	"regexp"
	"strconv"
	"strings"
)

// Accounts accepts 5-32 chars of alphanumerics and single internal hyphens.
// Derived aliases stop at 30 to leave room for a suffix.
const (
	aliasMinLen         = 5
	aliasMaxLen         = 30
	aliasHardMaxLen     = 32
	aliasCreateAttempts = 5
)

var (
	aliasUnsafe    = regexp.MustCompile(`[^a-zA-Z0-9-]`)
	aliasHyphenRun = regexp.MustCompile(`-+`)
)

// deriveAlias mirrors the transform accounts applies in its own alias migrations, for clients that cannot send one yet.
func deriveAlias(name string) string {
	slug := aliasUnsafe.ReplaceAllString(name, "-")
	slug = aliasHyphenRun.ReplaceAllString(slug, "-")
	slug = strings.ToLower(strings.Trim(slug, "-"))

	// A name with no ASCII leaves nothing to slug, so a random token keeps such aliases distinct.
	if slug == "" {
		return "w-" + randomAliasToken()
	}

	if len(slug) > aliasMaxLen {
		slug = strings.TrimRight(slug[:aliasMaxLen], "-")
	}
	for len(slug) < aliasMinLen {
		slug += "a"
	}
	return slug
}

// aliasCandidate suffixes a taken alias, trimming the base to stay within the allowed length.
func aliasCandidate(base string, attempt int) string {
	suffix := "-" + strconv.Itoa(attempt)
	if max := aliasHardMaxLen - len(suffix); len(base) > max {
		base = strings.TrimRight(base[:max], "-")
	}
	return base + suffix
}

// isDuplicateAliasErr matches on the message because the accounts gqlclient collapses every failure but 401 into an opaque error.
func isDuplicateAliasErr(err error) bool {
	return err != nil && strings.Contains(strings.ToLower(err.Error()), "alias already exists")
}

// randomAliasToken needs uniqueness, not unpredictability.
func randomAliasToken() string {
	return fmt.Sprintf("%016x", rand.Uint64())
}
