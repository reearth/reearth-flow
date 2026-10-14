package project

import (
	"errors"
	"regexp"
	"strings"
)

type Visibility string

const (
	VisibilityPublic  Visibility = "public"
	VisibilityPrivate Visibility = "private"

	// DefaultAliasPrefix marks the alias a project gets when none is chosen. An
	// alias with this prefix is reserved for the project whose ID follows it.
	DefaultAliasPrefix = "p-"

	MaxTopics      = 200
	MaxTopicLength = 64
)

var (
	ErrInvalidAlias      = errors.New("invalid alias: use 5-32 letters, digits, '-' or '_'")
	ErrReservedAlias     = errors.New("invalid alias: the 'p-' prefix is reserved")
	ErrInvalidVisibility = errors.New("invalid visibility")
	ErrTooManyTopics     = errors.New("too many topics")
	ErrTopicTooLong      = errors.New("topic is too long")

	aliasPattern = regexp.MustCompile(`^[a-zA-Z0-9_-]{5,32}$`)
)

func (v Visibility) Valid() bool {
	return v == VisibilityPublic || v == VisibilityPrivate
}

// DefaultAlias is the alias a project has until one is chosen. It is unique
// because the ID is.
func DefaultAlias(pid ID) string {
	return DefaultAliasPrefix + pid.String()
}

// ValidateAlias checks alias for the project pid. The default-alias prefix is
// accepted only as pid's own default alias, so a project cannot take the alias
// another project holds by default.
func ValidateAlias(alias string, pid ID) error {
	if !aliasPattern.MatchString(alias) {
		return ErrInvalidAlias
	}
	if strings.HasPrefix(alias, DefaultAliasPrefix) && alias != DefaultAlias(pid) {
		return ErrReservedAlias
	}
	return nil
}

// NormalizeTopics trims each topic and drops blanks and repeats, keeping the
// first occurrence's order.
func NormalizeTopics(topics []string) ([]string, error) {
	res := make([]string, 0, len(topics))
	seen := make(map[string]struct{}, len(topics))
	for _, t := range topics {
		t = strings.TrimSpace(t)
		if t == "" {
			continue
		}
		if len([]rune(t)) > MaxTopicLength {
			return nil, ErrTopicTooLong
		}
		if _, ok := seen[t]; ok {
			continue
		}
		seen[t] = struct{}{}
		res = append(res, t)
	}
	if len(res) > MaxTopics {
		return nil, ErrTooManyTopics
	}
	return res, nil
}
