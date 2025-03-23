package pubsub

import (
	"context"
	"encoding/json"
	"log"

	"github.com/reearth/reearth-flow/subscriber/internal/usecase/interactor"
	"github.com/reearth/reearth-flow/subscriber/pkg/node"
)

type NodeSubscriber struct {
	sub     Subscription
	useCase interactor.NodeSubscriberUseCase
}

func NewNodeSubscriber(subscription Subscription, useCase interactor.NodeSubscriberUseCase) *NodeSubscriber {
	return &NodeSubscriber{
		sub:     subscription,
		useCase: useCase,
	}
}

func (s *NodeSubscriber) StartListening(ctx context.Context) error {
	log.Println("[NodeSubscriber] Starting to listen for edge pass through events")

	return s.sub.Receive(ctx, func(ctx context.Context, m Message) {
		defer func() {
			if r := recover(); r != nil {
				log.Printf("[NodeSubscriber] panic recovered: %v", r)
			}
		}()

		var evt node.NodeStatusEvent
		if err := json.Unmarshal(m.Data(), &evt); err != nil {
			log.Printf("[NodeSubscriber] failed to unmarshal edge event: %v", err)
			m.Nack()
			return
		}

		if err := s.useCase.ProcessNodeEvent(ctx, &evt); err != nil {
			log.Printf("[NodeSubscriber] failed to process edge event: %v", err)
			m.Nack()
			return
		}

		m.Ack()
		log.Printf("[NodeSubscriber] Successfully processed event for job %s", evt.JobID)
	})
}
