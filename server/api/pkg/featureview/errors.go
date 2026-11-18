package featureview

import "errors"

var (
	// ErrInvalidRequest is a request the engine could not be asked to render.
	// Callers surface it to the user rather than logging it.
	ErrInvalidRequest = errors.New("invalid view request")
	ErrInvalidFileID  = errors.New("invalid intermediate-data file id")
	ErrInvalidViewKey = errors.New("invalid view key")
	// ErrInvalidReport is a report that could not be read, or that describes an
	// outcome inconsistently.
	ErrInvalidReport = errors.New("invalid view report")
)
