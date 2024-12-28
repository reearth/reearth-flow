package pubsub

import (
	"context"
	"encoding/json"
	"log"

	"cloud.google.com/go/pubsub"
	"github.com/reearth/reearth-flow/log-subscriber/internal/usecase/interactor"
	domainLog "github.com/reearth/reearth-flow/log-subscriber/pkg/log"
)

type Subscriber struct {
	subscription *pubsub.Subscription
	useCase      interactor.LogSubscriberUseCase
}

func NewSubscriber(subscription *pubsub.Subscription, useCase interactor.LogSubscriberUseCase) *Subscriber {
	return &Subscriber{
		subscription: subscription,
		useCase:      useCase,
	}
}

// Subscribe and receive messages
func (s *Subscriber) StartListening(ctx context.Context) error {
	return s.subscription.Receive(ctx, func(ctx context.Context, m *pubsub.Message) {

		// Automatically recovers when panic occurs
		defer func() {
			if r := recover(); r != nil {
				log.Printf("panic recovered in subscriber: %v", r)
			}
		}()
		var evt domainLog.LogEvent
		if err := json.Unmarshal(m.Data, &evt); err != nil {
			log.Printf("failed to unmarshal message: %v", err)
			m.Nack()
			return
		}

		// Process using LogSubscriberUseCase
		if err := s.useCase.ProcessLogEvent(ctx, &evt); err != nil {
			log.Printf("failed to process event: %v", err)
			m.Nack()
			return
		}

		m.Ack()
	})
}
