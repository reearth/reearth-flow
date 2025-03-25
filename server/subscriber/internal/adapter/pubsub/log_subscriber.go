package pubsub

import (
	"context"
	"encoding/json"
	"log"

	"github.com/reearth/reearth-flow/subscriber/internal/usecase/interactor"
	domainLog "github.com/reearth/reearth-flow/subscriber/pkg/log"
)

type LogSubscriber struct {
	sub     Subscription
	useCase interactor.LogSubscriberUseCase
}

func NewLogSubscriber(subscription Subscription, useCase interactor.LogSubscriberUseCase) *LogSubscriber {
	return &LogSubscriber{
		sub:     subscription,
		useCase: useCase,
	}
}

func (s *LogSubscriber) StartListening(ctx context.Context) error {
	return s.sub.Receive(ctx, func(ctx context.Context, m Message) {
		defer func() {
			if r := recover(); r != nil {
				log.Printf("[LogSubscriber] panic recovered: %v", r)
			}
		}()

		var evt domainLog.LogEvent
		if err := json.Unmarshal(m.Data(), &evt); err != nil {
			log.Printf("failed to unmarshal message: %v", err)
			m.Nack()
			return
		}

		if err := s.useCase.ProcessLogEvent(ctx, &evt); err != nil {
			log.Printf("failed to process event: %v", err)
			m.Nack()
			return
		}

		m.Ack()
	})
}
