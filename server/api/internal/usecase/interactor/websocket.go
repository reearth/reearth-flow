package interactor

import (
	"context"
	"fmt"
	"sync"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/websocket"
	"github.com/reearth/reearth-flow/api/pkg/document"
	"github.com/reearth/reearthx/log"
)

var (
	defaultWebsocketClient websocket.WebsocketClient
	wsClientMutex          sync.RWMutex
)

func InitWebsocket(serverURL string) {
	ctx := context.Background()
	config := websocket.WebsocketConfig{
		ServerURL: serverURL,
	}

	if err := InitWebsocketClient(ctx, config); err != nil {
		log.Errorf("websocket: failed to initialize client: %v", err)
	} else {
		log.Infof("websocket: client initialized successfully with server URL: %s", serverURL)
	}
}

func InitWebsocketClient(ctx context.Context, config websocket.WebsocketConfig) error {
	wsClientMutex.Lock()
	defer wsClientMutex.Unlock()

	if defaultWebsocketClient != nil {
		if err := defaultWebsocketClient.Close(); err != nil {
			log.Errorfc(ctx, "websocket: failed to close existing client: %v", err)
		}
	}

	client, err := websocket.NewDocumentRepo(ctx, config)
	if err != nil {
		return fmt.Errorf("failed to initialize websocket client: %w", err)
	}

	defaultWebsocketClient = client
	log.Debugfc(ctx, "websocket: default client initialized successfully")
	return nil
}

func CloseWebsocketClient() error {
	wsClientMutex.Lock()
	defer wsClientMutex.Unlock()

	if defaultWebsocketClient == nil {
		return nil
	}

	err := defaultWebsocketClient.Close()
	defaultWebsocketClient = nil
	return err
}

func getDefaultWebsocketClient() websocket.WebsocketClient {
	wsClientMutex.RLock()
	defer wsClientMutex.RUnlock()
	return defaultWebsocketClient
}

func GetLatest(ctx context.Context, id string) (*document.Document, error) {
	client := getDefaultWebsocketClient()
	if client == nil {
		return nil, fmt.Errorf("websocket client is not initialized")
	}
	return client.GetLatest(ctx, id)
}

func GetHistory(ctx context.Context, id string) ([]*document.History, error) {
	client := getDefaultWebsocketClient()
	if client == nil {
		return nil, fmt.Errorf("websocket client is not initialized")
	}
	return client.GetHistory(ctx, id)
}

func Rollback(ctx context.Context, id string, version int) (*document.Document, error) {
	client := getDefaultWebsocketClient()
	if client == nil {
		return nil, fmt.Errorf("websocket client is not initialized")
	}
	return client.Rollback(ctx, id, version)
}
