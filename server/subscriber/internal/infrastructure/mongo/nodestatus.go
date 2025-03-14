package mongo

import (
	"context"
	"fmt"
	"log"
	"time"

	"github.com/reearth/reearth-flow/subscriber/pkg/nodestatus"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
)

type MongoStorage struct {
	client        MongoClient
	jobCollection string
}

func NewMongoStorage(client MongoClient, jobCollection string) *MongoStorage {
	return &MongoStorage{
		client:        client,
		jobCollection: jobCollection,
	}
}

// getEdgeNodes gets the fromNode and toNode for a given edge ID
// This is needed since the worker doesn't provide this information
func (m *MongoStorage) getEdgeNodes(ctx context.Context, jobID, edgeID string) (fromNode, toNode string, err error) {
	jobCollection := m.client.Collection(m.jobCollection)
	
	// Find the job that contains the edge
	var job struct {
		Edges []struct {
			ID   string `bson:"id"`
			From string `bson:"from"`
			To   string `bson:"to"`
		} `bson:"edges"`
	}
	
	err = jobCollection.FindOne(ctx, bson.M{"_id": jobID}).Decode(&job)
	if err != nil {
		return "", "", fmt.Errorf("failed to find job: %w", err)
	}
	
	// Find the edge with the matching ID
	for _, edge := range job.Edges {
		if edge.ID == edgeID {
			return edge.From, edge.To, nil
		}
	}
	
	return "", "", fmt.Errorf("edge not found: %s", edgeID)
}

// findDownstreamNodeIDs finds all nodes that are downstream from the given node
func (m *MongoStorage) findDownstreamNodeIDs(ctx context.Context, jobID, nodeID string) ([]string, error) {
	collection := m.client.Collection(m.jobCollection)

	// Get job document to find edges
	var job struct {
		NodeExecutions []struct {
			NodeID string `bson:"nodeId"`
		} `bson:"nodeExecutions"`
		Edges []struct {
			From string `bson:"from"`
			To   string `bson:"to"`
		} `bson:"edges"`
	}

	err := collection.FindOne(ctx, bson.M{"_id": jobID}).Decode(&job)
	if err != nil {
		return nil, fmt.Errorf("failed to find job: %w", err)
	}

	// Build adjacency list for graph traversal
	adjacencyList := make(map[string][]string)
	for _, edge := range job.Edges {
		adjacencyList[edge.From] = append(adjacencyList[edge.From], edge.To)
	}

	// Perform BFS to find all downstream nodes
	downstream := make([]string, 0)
	queue := []string{nodeID}
	visited := make(map[string]bool)

	for len(queue) > 0 {
		current := queue[0]
		queue = queue[1:]

		if visited[current] {
			continue
		}
		visited[current] = true

		// Don't add the starting node to the downstream list
		if current != nodeID {
			downstream = append(downstream, current)
		}

		// Add all neighbors to queue
		for _, neighbor := range adjacencyList[current] {
			if !visited[neighbor] {
				queue = append(queue, neighbor)
			}
		}
	}

	return downstream, nil
}

// markDownstreamNodesAsSkipped marks all downstream nodes as skipped due to an upstream failure
func (m *MongoStorage) markDownstreamNodesAsSkipped(ctx context.Context, jobID, failedNodeID, reason string) error {
	// Find all downstream nodes
	downstreamNodes, err := m.findDownstreamNodeIDs(ctx, jobID, failedNodeID)
	if err != nil {
		// Log but don't fail - best effort
		log.Printf("Warning: Failed to find downstream nodes: %v\n", err)
		return nil
	}

	if len(downstreamNodes) == 0 {
		return nil
	}

	collection := m.client.Collection(m.jobCollection)
	now := time.Now()
	
	// Create a filter to match nodes in the downstream list
	filters := make([]interface{}, len(downstreamNodes))
	for i, nodeID := range downstreamNodes {
		filters[i] = bson.M{"elem.nodeId": nodeID}
	}

	// Update all downstream nodes to SKIPPED
	update := bson.M{
		"$set": bson.M{
			"nodeExecutions.$[elem].status":      string(nodestatus.NodeExecutionStatusSkipped),
			"nodeExecutions.$[elem].completedAt": now,
			"nodeExecutions.$[elem].error":       fmt.Sprintf("Skipped due to upstream failure: %s", reason),
		},
	}

	filter := options.Update().SetArrayFilters(options.ArrayFilters{
		Filters: []interface{}{
			bson.M{"$or": filters},
		},
	})

	_, err = collection.UpdateOne(
		ctx,
		bson.M{"_id": jobID},
		update,
		filter,
	)
	
	if err != nil {
		log.Printf("Warning: Failed to mark downstream nodes as skipped: %v\n", err)
	}
	
	return nil
}

// UpdateNodeExecutions updates node execution status in the job document based on edge pass events
func (m *MongoStorage) UpdateNodeExecutions(ctx context.Context, event *nodestatus.EdgePassThroughEvent) error {
	jobCollection := m.client.Collection(m.jobCollection)
	
	for _, edge := range event.UpdatedEdges {
		// First, get from/to nodes since worker doesn't provide them
		fromNode, toNode, err := m.getEdgeNodes(ctx, event.JobID, edge.ID)
		if err != nil {
			log.Printf("Warning: Failed to get nodes for edge %s: %v\n", edge.ID, err)
			continue
		}
		
		now := time.Now()
		
		// Handle edge pass events
		if edge.Status == nodestatus.EventStatusInProgress {
			// Update fromNode to SUCCEEDED
			updateFromNode := bson.M{
				"$set": bson.M{
					"nodeExecutions.$[elem].status":      string(nodestatus.NodeExecutionStatusSucceeded),
					"nodeExecutions.$[elem].completedAt": now,
				},
			}
			
			fromNodeFilter := options.Update().SetArrayFilters(options.ArrayFilters{
				Filters: []interface{}{bson.M{"elem.nodeId": fromNode}},
			})
			
			_, err := jobCollection.UpdateOne(
				ctx,
				bson.M{"_id": event.JobID},
				updateFromNode,
				fromNodeFilter,
			)
			if err != nil {
				log.Printf("Warning: Failed to update fromNode %s status: %v\n", fromNode, err)
			}
			
			// Update toNode to RUNNING
			updateToNode := bson.M{
				"$set": bson.M{
					"nodeExecutions.$[elem].status":    string(nodestatus.NodeExecutionStatusRunning),
					"nodeExecutions.$[elem].startedAt": now,
					"nodeExecutions.$[elem].error":     nil, // Clear any previous errors
				},
			}
			
			toNodeFilter := options.Update().SetArrayFilters(options.ArrayFilters{
				Filters: []interface{}{bson.M{"elem.nodeId": toNode}},
			})
			
			_, err = jobCollection.UpdateOne(
				ctx,
				bson.M{"_id": event.JobID},
				updateToNode,
				toNodeFilter,
			)
			if err != nil {
				log.Printf("Warning: Failed to update toNode %s status: %v\n", toNode, err)
			}
		} else if edge.Status == nodestatus.EventStatusCompleted {
			// Mark toNode as SUCCEEDED when edge is completed
			updateToNode := bson.M{
				"$set": bson.M{
					"nodeExecutions.$[elem].status":      string(nodestatus.NodeExecutionStatusSucceeded),
					"nodeExecutions.$[elem].completedAt": now,
				},
			}
			
			toNodeFilter := options.Update().SetArrayFilters(options.ArrayFilters{
				Filters: []interface{}{bson.M{"elem.nodeId": toNode}},
			})
			
			_, err := jobCollection.UpdateOne(
				ctx,
				bson.M{"_id": event.JobID},
				updateToNode,
				toNodeFilter,
			)
			if err != nil {
				log.Printf("Warning: Failed to update toNode %s status: %v\n", toNode, err)
			}
		}
	}
	
	return nil
}

// MarkNodeAsFailed marks a specific node as failed and skips downstream nodes
func (m *MongoStorage) MarkNodeAsFailed(ctx context.Context, jobID, nodeID, errorMessage string) error {
	jobCollection := m.client.Collection(m.jobCollection)
	now := time.Now()
	
	update := bson.M{
		"$set": bson.M{
			"nodeExecutions.$[elem].status":      string(nodestatus.NodeExecutionStatusFailed),
			"nodeExecutions.$[elem].completedAt": now,
			"nodeExecutions.$[elem].error":       errorMessage,
		},
	}
	
	filter := options.Update().SetArrayFilters(options.ArrayFilters{
		Filters: []interface{}{bson.M{"elem.nodeId": nodeID}},
	})
	
	_, err := jobCollection.UpdateOne(
		ctx,
		bson.M{"_id": jobID},
		update,
		filter,
	)
	if err != nil {
		if err == mongo.ErrNoDocuments {
			return fmt.Errorf("no job found with ID %s", jobID)
		}
		return fmt.Errorf("failed to mark node %s as failed: %w", nodeID, err)
	}
	
	// Mark all downstream nodes as SKIPPED
	err = m.markDownstreamNodesAsSkipped(ctx, jobID, nodeID, errorMessage)
	if err != nil {
		log.Printf("Warning: Failed to mark downstream nodes as skipped: %v\n", err)
	}
	
	return nil
}

// MarkJobAsFailed marks the job and all running nodes as failed, pending nodes as skipped
func (m *MongoStorage) MarkJobAsFailed(ctx context.Context, jobID, errorMessage string) error {
	jobCollection := m.client.Collection(m.jobCollection)
	now := time.Now()
	
	// First update the job status itself
	_, err := jobCollection.UpdateOne(
		ctx,
		bson.M{"_id": jobID},
		bson.M{
			"$set": bson.M{
				"status":      "FAILED",
				"completedAt": now,
				"error":       errorMessage,
			},
		},
	)
	
	if err != nil {
		log.Printf("Warning: Failed to update job status: %v\n", err)
	}
	
	// Update all RUNNING nodes to FAILED
	runningUpdate := bson.M{
		"$set": bson.M{
			"nodeExecutions.$[elem].status":      string(nodestatus.NodeExecutionStatusFailed),
			"nodeExecutions.$[elem].completedAt": now,
			"nodeExecutions.$[elem].error":       errorMessage,
		},
	}
	
	runningFilter := options.Update().SetArrayFilters(options.ArrayFilters{
		Filters: []interface{}{bson.M{"elem.status": string(nodestatus.NodeExecutionStatusRunning)}},
	})
	
	_, err = jobCollection.UpdateOne(
		ctx,
		bson.M{"_id": jobID},
		runningUpdate,
		runningFilter,
	)
	
	if err != nil {
		log.Printf("Warning: Failed to update running nodes to FAILED: %v\n", err)
	}
	
	// Update all PENDING nodes to SKIPPED
	pendingUpdate := bson.M{
		"$set": bson.M{
			"nodeExecutions.$[elem].status":      string(nodestatus.NodeExecutionStatusSkipped),
			"nodeExecutions.$[elem].completedAt": now,
			"nodeExecutions.$[elem].error":       fmt.Sprintf("Skipped due to job failure: %s", errorMessage),
		},
	}
	
	pendingFilter := options.Update().SetArrayFilters(options.ArrayFilters{
		Filters: []interface{}{bson.M{"elem.status": string(nodestatus.NodeExecutionStatusPending)}},
	})
	
	_, err = jobCollection.UpdateOne(
		ctx,
		bson.M{"_id": jobID},
		pendingUpdate,
		pendingFilter,
	)
	
	if err != nil {
		log.Printf("Warning: Failed to update pending nodes to SKIPPED: %v\n", err)
	}
	
	return nil
}
