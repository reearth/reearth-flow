package featureview

import "fmt"

// MaxFileIDLength bounds a file id so a hostile one cannot be used to build an
// unreasonable object name.
const MaxFileIDLength = 512

// ValidateFileID checks an intermediate-data file id before it is used to build
// a storage path.
//
// A file id is the engine's `[subgraphPrefix.]nodeID.port`, so dots are
// expected and carry meaning. Everything else is refused: the id is
// client-supplied and is used on the WRITE side, where path.Clean on the read
// side is no protection. Only a single path segment of the conservative
// character set the engine actually produces is accepted.
func ValidateFileID(fileID string) error {
	if fileID == "" {
		return fmt.Errorf("%w: must not be empty", ErrInvalidFileID)
	}
	if len(fileID) > MaxFileIDLength {
		return fmt.Errorf("%w: longer than the %d character maximum", ErrInvalidFileID, MaxFileIDLength)
	}
	if fileID[0] == '.' {
		return fmt.Errorf("%w: must not start with a dot", ErrInvalidFileID)
	}

	for i := 0; i < len(fileID); i++ {
		c := fileID[i]
		switch {
		case c >= 'a' && c <= 'z',
			c >= 'A' && c <= 'Z',
			c >= '0' && c <= '9',
			c == '.', c == '-', c == '_':
		default:
			return fmt.Errorf("%w: unexpected character %q", ErrInvalidFileID, string(c))
		}
		// A dot run cannot climb out of the directory here, since a separator is
		// already refused — but it also never appears in a real file id, so
		// refusing it keeps the accepted set exactly what the engine emits.
		if c == '.' && i+1 < len(fileID) && fileID[i+1] == '.' {
			return fmt.Errorf("%w: must not contain consecutive dots", ErrInvalidFileID)
		}
	}

	return nil
}
