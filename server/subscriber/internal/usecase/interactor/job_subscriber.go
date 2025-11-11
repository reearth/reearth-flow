package interactor

import (
	"context"
	"fmt"

	"github.com/reearth/reearth-flow/subscriber/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/subscriber/pkg/job"
)

type JobSubscriberUseCase interface {
	ProcessJobCompleteEvent(ctx context.Context, event *job.JobCompleteEvent) error
}

type jobSubscriber struct {
	storage gateway.JobStorage
}

func NewJobSubscriberUseCase(storage gateway.JobStorage) JobSubscriberUseCase {
	return &jobSubscriber{storage: storage}
}

func (uc *jobSubscriber) ProcessJobCompleteEvent(ctx context.Context, event *job.JobCompleteEvent) error {
	if event == nil {
		return fmt.Errorf("event is nil")
	}

	if event.Result != "success" && event.Result != "failed" {
		return fmt.Errorf("unknown result: %s (expected 'success' or 'failed')", event.Result)
	}

	if err := uc.storage.SaveToRedis(ctx, event); err != nil {
		return fmt.Errorf("failed to write to Redis: %w", err)
	}

	return nil
}
