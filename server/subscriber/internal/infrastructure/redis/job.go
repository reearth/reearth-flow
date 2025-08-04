package redis

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/redis/go-redis/v9"
	"github.com/reearth/reearth-flow/subscriber/pkg/job"
)

type JobStorageRedis struct {
	client *redis.Client
}

func NewJobStorageRedis(client *redis.Client) *JobStorageRedis {
	return &JobStorageRedis{
		client: client,
	}
}

func (r *JobStorageRedis) SaveToRedis(ctx context.Context, event *job.JobStatusEvent) error {
	key := fmt.Sprintf("job_status:%s", event.JobID)

	jobData := map[string]interface{}{
		"jobId":      event.JobID,
		"workflowId": event.WorkflowID,
		"status":     string(event.Status),
		"timestamp":  event.Timestamp,
	}

	if event.Message != nil {
		jobData["message"] = *event.Message
	}

	if event.FailedNodes != nil {
		jobData["failedNodes"] = *event.FailedNodes
	}

	data, err := json.Marshal(jobData)
	if err != nil {
		return fmt.Errorf("failed to marshal job status data: %w", err)
	}

	err = r.client.Set(ctx, key, data, 24*time.Hour).Err()
	if err != nil {
		return fmt.Errorf("failed to save job status to Redis: %w", err)
	}

	return nil
}
