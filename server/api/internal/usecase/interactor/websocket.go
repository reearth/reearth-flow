
	client := getDefaultWebsocketClient()
	if client == nil {
		return nil, fmt.Errorf("websocket client is not initialized")
	}
	return client.GetLatest(ctx, id)
}

func GetHistory(ctx context.Context, id string) ([]*doc.History, error) {
	client := getDefaultWebsocketClient()
	if client == nil {
		return nil, fmt.Errorf("websocket client is not initialized")
	}
	return client.GetHistory(ctx, id)
}

func Rollback(ctx context.Context, id string, version int) (*doc.Document, error) {
	client := getDefaultWebsocketClient()
	if client == nil {
		return nil, fmt.Errorf("websocket client is not initialized")
	}
	return client.Rollback(ctx, id, version)
}
