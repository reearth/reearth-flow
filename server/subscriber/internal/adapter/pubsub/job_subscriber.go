package pubsub

import (
	"context"
	"encoding/json"
	"log"

	"github.com/reearth/reearth-flow/subscriber/internal/usecase/interactor"
	domainJob "github.com/reearth/reearth-flow/subscriber/pkg/job"
)

type JobSubscriber struct {
	sub     Subscription
	useCase interactor.JobSubscriberUseCase
}

func NewJobSubscriber(subscription Subscription, useCase interactor.JobSubscriberUseCase) *JobSubscriber {
	return &JobSubscriber{
		sub:     subscription,
		useCase: useCase,
	}
}

func (s *JobSubscriber) StartListening(ctx context.Context) error {
	return s.sub.Receive(ctx, func(ctx context.Context, m Message) {
		defer func() {
			if r := recover(); r != nil {
				log.Printf("[JobSubscriber] panic recovered: %v", r)
			}
		}()

		var evt domainJob.JobCompleteEvent
		if err := json.Unmarshal(m.Data(), &evt); err != nil {
			log.Printf("failed to unmarshal message: %v", err)
			m.Nack()
			return
		}

		if err := s.useCase.ProcessJobCompleteEvent(ctx, &evt); err != nil {
			log.Printf("failed to process event: %v", err)
			m.Nack()
			return
		}

		m.Ack()
	})
}
