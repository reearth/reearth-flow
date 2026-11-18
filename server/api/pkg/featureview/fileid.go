package featureview

import (
	"fmt"
	"strings"
	"unicode"
	"unicode/utf8"
)

// MaxFileIDLength bounds a file id, in bytes, so a hostile one cannot be used
// to build an unreasonable object name.
const MaxFileIDLength = 512

// ValidateFileID checks a client-supplied file id before it becomes a storage
// path segment on the write side. Port names are author-chosen (`建物`,
// `high rise`), so only separators, control characters and a leading dot are
// refused.
func ValidateFileID(fileID string) error {
	if fileID == "" {
		return fmt.Errorf("%w: must not be empty", ErrInvalidFileID)
	}
	if len(fileID) > MaxFileIDLength {
		return fmt.Errorf("%w: longer than the %d byte maximum", ErrInvalidFileID, MaxFileIDLength)
	}
	if !utf8.ValidString(fileID) {
		return fmt.Errorf("%w: not valid UTF-8", ErrInvalidFileID)
	}
	if strings.HasPrefix(fileID, ".") {
		return fmt.Errorf("%w: must not start with a dot", ErrInvalidFileID)
	}

	for _, r := range fileID {
		switch {
		case r == '/', r == '\\':
			return fmt.Errorf("%w: must not contain a path separator", ErrInvalidFileID)
		case unicode.IsControl(r):
			return fmt.Errorf("%w: unexpected control character %U", ErrInvalidFileID, r)
		}
	}

	return nil
}
