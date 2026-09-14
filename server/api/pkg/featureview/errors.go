package featureview

import "errors"

var (
	// ErrInvalidRequest is a request the engine could not be asked to render.
	// Callers surface it to the user rather than logging it.
	ErrInvalidRequest = errors.New("invalid view request")
	// ErrInvalidFileID is a file id that could not be used as a storage path
	// segment.
	ErrInvalidFileID = errors.New("invalid intermediate-data file id")
	// ErrInvalidReport is a report that could not be read, or that describes an
	// outcome inconsistently.
	ErrInvalidReport = errors.New("invalid view report")
)
