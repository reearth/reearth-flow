package document

import (
	"context"
	"sync"
	"time"

	"github.com/reearth/reearthx/log"
)

var (
	defaultClient *Client
	clientMu      sync.RWMutex
)

func getDefaultClient() *Client {
	clientMu.RLock()
	if defaultClient != nil {
		defer clientMu.RUnlock()
		return defaultClient
	}
	clientMu.RUnlock()

	clientMu.Lock()
	defer clientMu.Unlock()
	if defaultClient == nil {
		log.Infof("Creating new document client with WebSocket URL: %s", config.WebsocketServerURL)
		defaultClient = NewClient(config.WebsocketServerURL)
	}
	return defaultClient
}

type Document struct {
	ID        string
	Update    []int
	Clock     int
	Timestamp time.Time
}

type History struct {
	Update    []int
	Clock     int
	Timestamp time.Time
}

func GetLatest(ctx context.Context, id string) (*Document, error) {
	return getDefaultClient().GetLatest(ctx, id)
}

func GetHistory(ctx context.Context, id string) ([]*History, error) {
	return getDefaultClient().GetHistory(ctx, id)
}
