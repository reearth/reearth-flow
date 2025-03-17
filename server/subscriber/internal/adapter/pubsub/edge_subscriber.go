package pubsub

import (
	"context"
	"encoding/json"
	"log"

	"github.com/reearth/reearth-flow/subscriber/internal/usecase/interactor"
	"github.com/reearth/reearth-flow/subscriber/pkg/edge"
)

type EdgeSubscriber struct {
	sub     Subscription
	useCase interactor.EdgeSubscriberUseCase
}

func NewEdgeSubscriber(subscription Subscription, useCase interactor.EdgeSubscriberUseCase) *EdgeSubscriber {
	return &EdgeSubscriber{
		sub:     subscription,
		useCase: useCase,
	}
}

func (s *EdgeSubscriber) StartListening(ctx context.Context) error {
	log.Println("[EdgeSubscriber] Starting to listen for edge pass through events")

	return s.sub.Receive(ctx, func(ctx context.Context, m Message) {
		defer func() {
			if r := recover(); r != nil {
				log.Printf("[EdgeSubscriber] panic recovered: %v", r)
			}
		}()

		var evt edge.PassThroughEvent
		if err := json.Unmarshal(m.Data(), &evt); err != nil {
			log.Printf("[EdgeSubscriber] failed to unmarshal edge event: %v", err)
			m.Nack()
			return
		}

		log.Printf("[EdgeSubscriber] Processing event for job %s with %d edge updates",
			evt.JobID, len(evt.UpdatedEdges))

		if err := s.useCase.ProcessEdgeEvent(ctx, &evt); err != nil {
			log.Printf("[EdgeSubscriber] failed to process edge event: %v", err)
			m.Nack()
			return
		}

		m.Ack()
		log.Printf("[EdgeSubscriber] Successfully processed event for job %s", evt.JobID)
	})
}
