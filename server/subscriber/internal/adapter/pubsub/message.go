package pubsub

import (
	"cloud.google.com/go/pubsub"
)

type Message interface {
	Data() []byte
	Ack()
	Nack()
}

type realMessage struct {
	msg *pubsub.Message
}

func NewRealMessage(m *pubsub.Message) Message {
	if m == nil {
		return nil
	}
	return &realMessage{msg: m}
}

func (r *realMessage) Data() []byte {
	return r.msg.Data
}

func (r *realMessage) Ack() {
	r.msg.Ack()
}

func (r *realMessage) Nack() {
	r.msg.Nack()
}
