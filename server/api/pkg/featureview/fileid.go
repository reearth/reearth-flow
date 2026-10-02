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

// ValidateFileID checks an intermediate-data file id before it is used to build
// a storage path.
//
// A file id is the engine's `[subgraphPrefix.]nodeID.port`, and the port part
// is a name the workflow author chose: a filter's output port can be `建物` or
// `high rise`. So the id is not held to a character set. It is refused only
// for what would make it unsafe as a single object-name segment: it is
// client-supplied and is used on the WRITE side, where path.Clean on the read
// side is no protection. Without a separator, a dot run cannot climb out of
// the view's directory; a leading dot is refused so that "." and ".." are too.
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
