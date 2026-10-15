package internalapi

import (
	"context"
	"errors"
	"fmt"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/rerror"
	"github.com/stretchr/testify/assert"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

func TestToStatus(t *testing.T) {
	tests := []struct {
		err  error
		name string
		want codes.Code
	}{
		{rerror.ErrNotFound, "not found", codes.NotFound},
		{fmt.Errorf("loading: %w", rerror.ErrNotFound), "wrapped not found", codes.NotFound},
		{interfaces.ErrOperationDenied, "denied", codes.PermissionDenied},
		{interfaces.ErrProjectAliasAlreadyUsed, "alias taken", codes.AlreadyExists},
		{interfaces.ErrPrivateProjectNotAllowed, "private not allowed", codes.FailedPrecondition},
		{project.ErrInvalidAlias, "invalid alias", codes.InvalidArgument},
		{project.ErrReservedAlias, "reserved alias", codes.InvalidArgument},
		{project.ErrInvalidVisibility, "invalid visibility", codes.InvalidArgument},
		{project.ErrTooManyTopics, "too many topics", codes.InvalidArgument},
		{project.ErrTopicTooLong, "topic too long", codes.InvalidArgument},
		{status.Error(codes.Aborted, "aborted"), "already a status", codes.Aborted},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			assert.Equal(t, tt.want, status.Code(toStatus(context.Background(), tt.err)))
		})
	}

	assert.NoError(t, toStatus(context.Background(), nil))
}

func TestToStatus_HidesUnknownErrors(t *testing.T) {
	err := toStatus(context.Background(), errors.New(`pq: relation "projects" does not exist`))
	s, _ := status.FromError(err)
	assert.Equal(t, codes.Internal, s.Code())
	assert.Equal(t, "internal error", s.Message(), "the database's text never reaches the caller")
}

func TestHiddenToNotFound(t *testing.T) {
	assert.ErrorIs(t, hiddenToNotFound(interfaces.ErrOperationDenied), rerror.ErrNotFound)
	other := errors.New("boom")
	assert.Equal(t, other, hiddenToNotFound(other))
	assert.NoError(t, hiddenToNotFound(nil))
}
