package interactor

import (
	"context"
	"fmt"

	"github.com/reearth/reearth-flow/subscriber/internal/usecase/gateway"
	domainLog "github.com/reearth/reearth-flow/subscriber/pkg/log"
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

func (u *logSubscriberUseCase) ProcessLogEvent(ctx context.Context, event *domainLog.LogEvent) error {
	if event == nil {
		return fmt.Errorf("event is nil")
	}
	if err := u.storage.SaveToRedis(ctx, event); err != nil {
		return fmt.Errorf("failed to write to Redis: %w", err)
	}
	return nil
}
