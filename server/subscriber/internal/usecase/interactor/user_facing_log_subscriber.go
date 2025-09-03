package interactor

import (
	"context"
	"fmt"

	"github.com/reearth/reearth-flow/subscriber/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/subscriber/pkg/userfacinglog"
)

type UserFacingLogSubscriberUseCase interface {
	ProcessUserFacingLogEvent(ctx context.Context, event *userfacinglog.UserFacingLogEvent) error
}

type userFacingLogSubscriberUseCase struct {
	storage gateway.UserFacingLogStorage
}

func NewUserFacingLogSubscriberUseCase(storage gateway.UserFacingLogStorage) UserFacingLogSubscriberUseCase {
	return &userFacingLogSubscriberUseCase{
		storage: storage,
	}
}

func (u *userFacingLogSubscriberUseCase) ProcessUserFacingLogEvent(ctx context.Context, event *userfacinglog.UserFacingLogEvent) error {
	if event == nil {
		return fmt.Errorf("user facing log event is nil")
	}

	// Validate required fields
	if event.WorkflowID == "" || event.JobID == "" {
		return fmt.Errorf("invalid event: missing workflow ID or job ID")
	}

	if err := u.storage.SaveToRedis(ctx, event); err != nil {
		return fmt.Errorf("failed to write user facing log to Redis: %w", err)
	}

	return nil
}
