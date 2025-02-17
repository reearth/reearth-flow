package websocket

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/websocket"
	pb "github.com/reearth/reearth-flow/api/proto"
	"github.com/reearth/reearthx/log"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

type Client struct {
	conn   *grpc.ClientConn
	client pb.DocumentServiceClient
}

func NewClient(address string) (*Client, error) {
	// If no port is specified, use the default gRPC port
	if !strings.Contains(address, ":") {
		address = fmt.Sprintf("%s:50051", address)
	}

	conn, err := grpc.Dial(address, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		return nil, fmt.Errorf("failed to connect to gRPC server: %w", err)
	}

	client := pb.NewDocumentServiceClient(conn)
	log.Infof("Created new document client with gRPC address: %s", address)
	return &Client{
		conn:   conn,
		client: client,
	}, nil
}

func (c *Client) Close() error {
	if c.conn != nil {
		return c.conn.Close()
	}
	return nil
}

func (c *Client) GetLatest(ctx context.Context, docID string) (*websocket.Document, error) {
	resp, err := c.client.GetLatestDocument(ctx, &pb.DocumentRequest{
		DocId: docID,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to get latest document: %w", err)
	}

	// Convert bytes to []int
	update := make([]int, len(resp.Content))
	for i, b := range resp.Content {
		update[i] = int(b)
	}

	return &websocket.Document{
		ID:        docID,
		Update:    update,
		Clock:     int(resp.Clock),
		Timestamp: time.Now(),
	}, nil
}

func (c *Client) GetHistory(ctx context.Context, docID string) ([]*websocket.History, error) {
	resp, err := c.client.GetDocumentHistory(ctx, &pb.DocumentHistoryRequest{
		DocId: docID,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to get document history: %w", err)
	}

	history := make([]*websocket.History, len(resp.Versions))
	for i, version := range resp.Versions {
		timestamp, err := time.Parse("2006-01-02 15:04:05.999999 -07:00:00", version.Timestamp)
		if err != nil {
			timestamp, err = time.Parse(time.RFC3339, version.Timestamp)
			if err != nil {
				log.Errorf("failed to parse timestamp: %v", err)
				timestamp = time.Time{}
			}
		}

		// Convert bytes to []int
		update := make([]int, len(version.Content))
		for j, b := range version.Content {
			update[j] = int(b)
		}

		history[i] = &websocket.History{
			Update:    update,
			Clock:     int(version.Clock),
			Timestamp: timestamp,
		}
	}

	return history, nil
}

func (c *Client) Rollback(ctx context.Context, id string, clock int) (*websocket.Document, error) {
	resp, err := c.client.RollbackDocument(ctx, &pb.RollbackRequest{
		DocId:     id,
		VersionId: fmt.Sprintf("%d", clock),
	})
	if err != nil {
		return nil, fmt.Errorf("failed to rollback document: %w", err)
	}

	if !resp.Success {
		return nil, fmt.Errorf("rollback failed: %s", resp.Message)
	}

	return c.GetLatest(ctx, id)
}
