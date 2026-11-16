package gql

import (
	"errors"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
)

// errorCodes are the errors a client tells apart by code rather than by
// message, sent as the GraphQL error's `extensions.code`. A message is written
// for people and may change; a code is a contract and does not.
//
// The first match wins, and an error is matched however deeply it is wrapped,
// since the interactors add detail by wrapping.
var errorCodes = []struct {
	err  error
	code string
}{
	{interfaces.ErrOperationDenied, "OPERATION_DENIED"},
	// The repos refuse with their own sentinel, which reads the same.
	{repo.ErrOperationDenied, "OPERATION_DENIED"},
	{interfaces.ErrJobNotFinished, "JOB_NOT_FINISHED"},
	{interfaces.ErrDeploymentHasTriggers, "DEPLOYMENT_HAS_TRIGGERS"},
	{interfaces.ErrIntermediateDataNotFound, "INTERMEDIATE_DATA_NOT_FOUND"},
	{interfaces.ErrViewsUnavailable, "VIEWS_UNAVAILABLE"},
	{interfaces.ErrRenderTimedOut, "VIEW_RENDER_TIMED_OUT"},
	{interfaces.ErrViewWorkersBusy, "VIEW_WORKERS_BUSY"},
}

// ErrorCode is the code a client receives for err, or "" for an error that has
// none.
func ErrorCode(err error) string {
	for _, c := range errorCodes {
		if errors.Is(err, c.err) {
			return c.code
		}
	}
	return ""
}
