package monitor

import "context"

type Config struct {
	Cancel          context.CancelFunc
	NotificationURL *string
}

type Registry interface {
	Register(jobID string, config *Config)
	Get(jobID string) *Config
	Remove(jobID string)
}
