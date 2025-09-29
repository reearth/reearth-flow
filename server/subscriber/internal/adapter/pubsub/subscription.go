package pubsub

import (
	"context"

	"cloud.google.com/go/pubsub/v2"
)

type Subscription interface {
	Receive(ctx context.Context, f func(context.Context, Message)) error
}

type realSubscription struct {
	sub *pubsub.Subscriber
}

func NewRealSubscription(sub *pubsub.Subscriber) Subscription {
	return &realSubscription{sub: sub}
}

func (r *realSubscription) Receive(ctx context.Context, f func(context.Context, Message)) error {
	return r.sub.Receive(ctx, func(ctx context.Context, m *pubsub.Message) {
		f(ctx, NewRealMessage(m))
	})
}
