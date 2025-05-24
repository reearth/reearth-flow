package interactor

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/subscriber/pkg/stdoutlog"
)

type StdoutLogUseCase interface {
	Process(ctx context.Context, event *stdoutlog.Event) error
}

type stdoutLogSubscriberUseCaseImpl struct {
	stdoutLogStorage gateway.StdoutLogStorage
}

func NewStdoutLogUseCase(storage gateway.StdoutLogStorage) StdoutLogUseCase {
	return &stdoutLogSubscriberUseCaseImpl{stdoutLogStorage: storage}
}

func (uc *stdoutLogSubscriberUseCaseImpl) Process(ctx context.Context, event *stdoutlog.Event) error {
	return uc.stdoutLogStorage.SaveToRedis(ctx, event)
}
