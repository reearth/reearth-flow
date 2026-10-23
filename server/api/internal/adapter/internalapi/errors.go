package internalapi

import (
	"context"
	"errors"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/rerror"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

var invalidArgumentErrors = []error{
	project.ErrInvalidAlias,
	project.ErrReservedAlias,
	project.ErrInvalidVisibility,
	project.ErrTooManyTopics,
	project.ErrTopicTooLong,
}

// toStatus turns a usecase error into a gRPC status. Errors it does not know
// are logged and reported as INTERNAL without their text, which may describe
// the database.
func toStatus(ctx context.Context, err error) error {
	if err == nil {
		return nil
	}
	if _, ok := status.FromError(err); ok {
		return err
	}
	switch {
	case errors.Is(err, rerror.ErrNotFound):
		return status.Error(codes.NotFound, "not found")
	case errors.Is(err, interfaces.ErrOperationDenied):
		return status.Error(codes.PermissionDenied, err.Error())
	case errors.Is(err, interfaces.ErrProjectAliasAlreadyUsed):
		return status.Error(codes.AlreadyExists, err.Error())
	case errors.Is(err, interfaces.ErrPrivateProjectNotAllowed):
		return status.Error(codes.FailedPrecondition, err.Error())
	}
	for _, e := range invalidArgumentErrors {
		if errors.Is(err, e) {
			return status.Error(codes.InvalidArgument, err.Error())
		}
	}
	log.Errorfc(ctx, "internalapi: %v", err)
	return status.Error(codes.Internal, "internal error")
}

// hiddenToNotFound reports a denied read as NOT_FOUND, so a caller cannot
// tell an item they may not see from one that does not exist.
func hiddenToNotFound(err error) error {
	if errors.Is(err, interfaces.ErrOperationDenied) {
		return rerror.ErrNotFound
	}
	return err
}

func invalidArgument(msg string) error {
	return status.Error(codes.InvalidArgument, msg)
}
