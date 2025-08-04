package pubsub

import (
	"context"
	"encoding/json"
	"log"

	"github.com/reearth/reearth-flow/subscriber/internal/usecase/interactor"
	"github.com/reearth/reearth-flow/subscriber/pkg/job"
)

type JobStatusSubscriber struct {
	sub     Subscription
	useCase interactor.JobSubscriberUseCase
}

func NewJobStatusSubscriber(subscription Subscription, useCase interactor.JobSubscriberUseCase) *JobStatusSubscriber {
	return &JobStatusSubscriber{
		sub:     subscription,
		useCase: useCase,
	}
}

func (s *JobStatusSubscriber) StartListening(ctx context.Context) error {
	log.Println("[JobStatusSubscriber] Starting to listen for job status events")

	return s.sub.Receive(ctx, func(ctx context.Context, m Message) {
		defer func() {
			if r := recover(); r != nil {
				log.Printf("[JobStatusSubscriber] panic recovered: %v", r)
			}
		}()

		var evt job.JobStatusEvent
		if err := json.Unmarshal(m.Data(), &evt); err != nil {
			log.Printf("[JobStatusSubscriber] failed to unmarshal job status event: %v", err)
			m.Nack()
			return
		}

		if err := s.useCase.ProcessJobStatusEvent(ctx, &evt); err != nil {
			log.Printf("[JobStatusSubscriber] failed to process job status event: %v", err)
			m.Nack()
			return
		}

		m.Ack()
		log.Printf("[JobStatusSubscriber] Successfully processed job status event for job %s with status %s",
			evt.JobID, evt.Status)
	})
}
