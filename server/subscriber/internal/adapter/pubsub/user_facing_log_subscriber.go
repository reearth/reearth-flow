package pubsub

import (
	"context"
	"encoding/json"
	"log"

	"github.com/reearth/reearth-flow/subscriber/internal/usecase/interactor"
	"github.com/reearth/reearth-flow/subscriber/pkg/userfacinglog"
)

type UserFacingLogSubscriber struct {
	sub     Subscription
	useCase interactor.UserFacingLogSubscriberUseCase
}

func NewUserFacingLogSubscriber(subscription Subscription, useCase interactor.UserFacingLogSubscriberUseCase) *UserFacingLogSubscriber {
	return &UserFacingLogSubscriber{
		sub:     subscription,
		useCase: useCase,
	}
}

func (s *UserFacingLogSubscriber) StartListening(ctx context.Context) error {
	return s.sub.Receive(ctx, func(ctx context.Context, m Message) {
		defer func() {
			if r := recover(); r != nil {
				log.Printf("[UserFacingLogSubscriber] panic recovered: %v", r)
			}
		}()

		var evt userfacinglog.UserFacingLogEvent
		if err := json.Unmarshal(m.Data(), &evt); err != nil {
			log.Printf("[UserFacingLogSubscriber] failed to unmarshal message: %v", err)
			m.Nack()
			return
		}

		// Log the received event for debugging
		nodeName := "<none>"
		if evt.NodeName != nil {
			nodeName = *evt.NodeName
		}
		log.Printf("[UserFacingLogSubscriber] received event: workflow=%s, job=%s, level=%s, node=%s",
			evt.WorkflowID, evt.JobID, evt.Level, nodeName)

		if err := s.useCase.ProcessUserFacingLogEvent(ctx, &evt); err != nil {
			log.Printf("[UserFacingLogSubscriber] failed to process user facing log event: %v", err)
			m.Nack()
			return
		}

		m.Ack()
	})
}
