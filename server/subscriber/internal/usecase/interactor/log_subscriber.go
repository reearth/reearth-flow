package interactor

import (
	"context"
	"fmt"
	"regexp"
	"strings"

	"github.com/reearth/reearth-flow/subscriber/internal/usecase/gateway"
	domainLog "github.com/reearth/reearth-flow/subscriber/pkg/log"
)

type LogSubscriberUseCase interface {
	ProcessLogEvent(ctx context.Context, event *domainLog.LogEvent) error
}

type logSubscriberUseCase struct {
	storage         gateway.LogStorage
	nodeStatusStore gateway.NodeStatusStorage
}

func NewLogSubscriberUseCase(storage gateway.LogStorage, nodeStatusStore gateway.NodeStatusStorage) LogSubscriberUseCase {
	return &logSubscriberUseCase{
		storage:         storage,
		nodeStatusStore: nodeStatusStore,
	}
}

var (
	nodeErrorRegex = regexp.MustCompile(`(?i)(node|action) ['"]?([^'"]+)['"]? (failed|error)`)
	jobErrorRegex  = regexp.MustCompile(`(?i)(job|workflow|execution) ['"]?([^'"]+)['"]? (failed|error)`)
)

func (u *logSubscriberUseCase) ProcessLogEvent(ctx context.Context, event *domainLog.LogEvent) error {
	if event == nil {
		return fmt.Errorf("event is nil")
	}
	
	if err := u.storage.SaveToRedis(ctx, event); err != nil {
		fmt.Printf("Warning: Failed to save log event to Redis: %v\n", err)
	}
	
	if u.nodeStatusStore == nil {
		return nil
	}
	
	if event.LogLevel == domainLog.LogLevelError {
		if event.NodeID != nil {
			err := u.nodeStatusStore.MarkNodeAsFailed(ctx, event.JobID, *event.NodeID, event.Message)
			if err != nil {
				fmt.Printf("Warning: Error marking node %s as failed: %v\n", *event.NodeID, err)
			}
			return nil
		}
		
		nodeMatches := nodeErrorRegex.FindStringSubmatch(event.Message)
		if len(nodeMatches) >= 3 {
			nodeID := nodeMatches[2]
			err := u.nodeStatusStore.MarkNodeAsFailed(ctx, event.JobID, nodeID, event.Message)
			if err != nil {
				fmt.Printf("Warning: Error marking node %s as failed: %v\n", nodeID, err)
			}
			return nil
		}
		
		if strings.Contains(strings.ToLower(event.Message), "job failed") || 
		   strings.Contains(strings.ToLower(event.Message), "workflow failed") || 
		   strings.Contains(strings.ToLower(event.Message), "execution failed") ||
		   jobErrorRegex.MatchString(event.Message) {
			
			err := u.nodeStatusStore.MarkJobAsFailed(ctx, event.JobID, event.Message)
			if err != nil {
				fmt.Printf("Warning: Error marking job %s as failed: %v\n", event.JobID, err)
			}
			return nil
		}
		
		err := u.nodeStatusStore.MarkJobAsFailed(ctx, event.JobID, 
			fmt.Sprintf("Job failed with error: %s", event.Message))
		if err != nil {
			fmt.Printf("Warning: Error marking job %s as failed: %v\n", event.JobID, err)
		}
	}
	
	return nil
}
