package interactor

import (
	"context"
	"fmt"
	"sync"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/websocket"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	ws "github.com/reearth/reearth-flow/api/pkg/websocket"
	"github.com/reearth/reearthx/log"
)

var (
	defaultClient interfaces.WebsocketClient
	clientConfig  websocket.Config
	clientOnce    sync.Once
)

func InitWebsocket(websocketThriftServerURL string) {
	clientConfig = websocket.Config{
		ServerURL: websocketThriftServerURL,
	}
}

func getDefaultWebsocketClient() interfaces.WebsocketClient {
	clientOnce.Do(func() {
		client, err := websocket.NewClient(clientConfig)
		if err != nil {
			log.Errorf("Failed to create websocket client: %v", err)
			return
		}
		defaultClient = client
	})
	if defaultClient == nil {
		log.Error("Websocket client is not initialized")
	}
	return defaultClient
}

func GetLatest(ctx context.Context, id string) (*ws.Document, error) {
	client := getDefaultWebsocketClient()
	if client == nil {
		return nil, fmt.Errorf("websocket client is not initialized")
	}
	return client.GetLatest(ctx, id)
}

func GetHistory(ctx context.Context, id string) ([]*ws.History, error) {
	client := getDefaultWebsocketClient()
	if client == nil {
		return nil, fmt.Errorf("websocket client is not initialized")
	}
	return client.GetHistory(ctx, id)
}

func GetHistoryMetadata(ctx context.Context, id string) ([]*ws.HistoryMetadata, error) {
	client := getDefaultWebsocketClient()
	if client == nil {
		return nil, fmt.Errorf("websocket client is not initialized")
	}
	return client.GetHistoryMetadata(ctx, id)
}

func Rollback(ctx context.Context, id string, version int) (*ws.Document, error) {
	client := getDefaultWebsocketClient()
	if client == nil {
		return nil, fmt.Errorf("websocket client is not initialized")
	}
	return client.Rollback(ctx, id, version)
}
