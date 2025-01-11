package interactor

import (
	"context"
	"fmt"

	"github.com/reearth/reearth-flow/log-subscriber/internal/usecase/gateway"
	domainLog "github.com/reearth/reearth-flow/log-subscriber/pkg/log"
)

type LogSubscriberUseCase interface {
	ProcessLogEvent(ctx context.Context, event *domainLog.LogEvent) error
}

type logSubscriberUseCase struct {
	storage gateway.LogStorage
}

func NewLogSubscriberUseCase(storage gateway.LogStorage) LogSubscriberUseCase {
	return &logSubscriberUseCase{
		storage: storage,
	}
}

// Save LogEvents received from Pub/Sub to Redis and GCS
func (u *logSubscriberUseCase) ProcessLogEvent(ctx context.Context, event *domainLog.LogEvent) error {
	if event == nil {
		return fmt.Errorf("event is nil")
	}

	// 1. Write to Redis
	if err := u.storage.SaveToRedis(ctx, event); err != nil {
		return fmt.Errorf("failed to write to Redis: %w", err)
	}

	// 2. Write to GCS
	if err := u.storage.SaveToGCS(ctx, event); err != nil {
		return fmt.Errorf("failed to write to GCS: %w", err)
	}

	return nil
}
