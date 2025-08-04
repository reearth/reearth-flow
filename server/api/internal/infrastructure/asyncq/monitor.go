package asyncq

import (
	"context"
	"sync"
	"time"

	"github.com/hibiken/asynq"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/notification"
	"github.com/reearth/reearth-flow/api/pkg/subscription"
	"github.com/reearth/reearthx/log"
)

type AsyncqMonitor struct {
	inspector      *asynq.Inspector
	jobRepo        repo.Job
	fileGateway    gateway.File
	notifier       notification.Notifier
	subscriptions  *subscription.JobManager
	config         *Config
	activeMonitors map[string]*MonitorConfig
	mu             sync.RWMutex
}

type MonitorConfig struct {
	JobID           string
	NotificationURL *string
	CreatedAt       time.Time
	LastChecked     time.Time
	Cancel          context.CancelFunc
}

func NewAsyncqMonitor(
	config *Config,
	jobRepo repo.Job,
	fileGateway gateway.File,
	notifier notification.Notifier,
	subscriptions *subscription.JobManager,
) *AsyncqMonitor {
	redisOpt := config.GetRedisClientOpt()
	inspector := asynq.NewInspector(redisOpt)

	return &AsyncqMonitor{
		inspector:      inspector,
		jobRepo:        jobRepo,
		fileGateway:    fileGateway,
		notifier:       notifier,
		subscriptions:  subscriptions,
		config:         config,
		activeMonitors: make(map[string]*MonitorConfig),
	}
}

func (m *AsyncqMonitor) StartMonitoring(ctx context.Context, j *job.Job, notificationURL *string) error {
	jobID := j.ID().String()

	m.mu.Lock()
	defer m.mu.Unlock()

	if _, exists := m.activeMonitors[jobID]; exists {
		log.Debugf("Job %s is already being monitored", jobID)
		if notificationURL != nil {
			m.activeMonitors[jobID].NotificationURL = notificationURL
		}
		return nil
	}

	monitorCtx, cancel := context.WithCancel(context.Background())

	config := &MonitorConfig{
		JobID:           jobID,
		NotificationURL: notificationURL,
		CreatedAt:       time.Now(),
		LastChecked:     time.Now(),
		Cancel:          cancel,
	}

	m.activeMonitors[jobID] = config

	go m.monitorJob(monitorCtx, j, config)

	log.Infof("Started monitoring job %s", jobID)
	return nil
}

func (m *AsyncqMonitor) StopMonitoring(jobID string) {
	m.mu.Lock()
	defer m.mu.Unlock()

	if config, exists := m.activeMonitors[jobID]; exists {
		config.Cancel()
		delete(m.activeMonitors, jobID)
		log.Infof("Stopped monitoring job %s", jobID)
	}
}

func (m *AsyncqMonitor) monitorJob(ctx context.Context, j *job.Job, config *MonitorConfig) {
	ticker := time.NewTicker(5 * time.Second)
	defer ticker.Stop()
	defer m.StopMonitoring(config.JobID)

	maxDuration := 24 * time.Hour

	for {
		select {
		case <-ctx.Done():
			log.Infof("Job monitoring cancelled for job %s", config.JobID)
			return
		case <-ticker.C:
			if time.Since(config.CreatedAt) > maxDuration {
				log.Warnf("Maximum monitoring duration exceeded for job %s", config.JobID)
				return
			}

			log.Debugf("AsyncQ Monitor heartbeat for job %s (event-driven mode)", config.JobID)

			config.LastChecked = time.Now()
		}
	}
}

func (m *AsyncqMonitor) handleJobCompletion(ctx context.Context, j *job.Job, config *MonitorConfig) error {
	if err := m.updateJobArtifacts(ctx, j); err != nil {
		log.Errorf("Failed to update job artifacts for %s: %v", config.JobID, err)
	}

	if err := m.jobRepo.Save(ctx, j); err != nil {
		return err
	}

	if config.NotificationURL != nil && *config.NotificationURL != "" {
		if err := m.sendCompletionNotification(ctx, j, *config.NotificationURL); err != nil {
			log.Errorf("Failed to send completion notification for %s: %v", config.JobID, err)
		}
	}

	return nil
}

func (m *AsyncqMonitor) updateJobArtifacts(ctx context.Context, j *job.Job) error {
	jobID := j.ID().String()

	outputs, err := m.fileGateway.ListJobArtifacts(ctx, jobID)
	if err != nil {
		return err
	}
	j.SetOutputURLs(outputs)

	logURL := m.fileGateway.GetJobLogURL(jobID)
	if logURL != "" {
		j.SetLogsURL(logURL)
	}

	workerLogURL := m.fileGateway.GetJobWorkerLogURL(jobID)
	if workerLogURL != "" {
		j.SetWorkerLogsURL(workerLogURL)
	}

	return nil
}

func (m *AsyncqMonitor) sendCompletionNotification(ctx context.Context, j *job.Job, notificationURL string) error {
	jobID := j.ID().String()

	status := "failed"
	switch j.Status() {
	case job.StatusCompleted:
		status = "succeeded"
	case job.StatusCancelled:
		status = "cancelled"
	}

	var logs []string
	if logExists, err := m.fileGateway.CheckJobLogExists(ctx, jobID); err == nil && logExists {
		logs = append(logs, j.LogsURL())
	}

	if workerLogExists, err := m.fileGateway.CheckJobWorkerLogExists(ctx, jobID); err == nil && workerLogExists {
		logs = append(logs, j.WorkerLogsURL())
	}

	payload := notification.Payload{
		RunID:        jobID,
		DeploymentID: j.Deployment().String(),
		Status:       status,
		Logs:         logs,
		Outputs:      j.OutputURLs(),
	}

	return m.notifier.Send(notificationURL, payload)
}

func (m *AsyncqMonitor) Close() error {
	m.mu.Lock()
	defer m.mu.Unlock()

	for jobID, config := range m.activeMonitors {
		config.Cancel()
		delete(m.activeMonitors, jobID)
	}

	return m.inspector.Close()
}

func (m *AsyncqMonitor) GetActiveMonitors() map[string]*MonitorConfig {
	m.mu.RLock()
	defer m.mu.RUnlock()

	result := make(map[string]*MonitorConfig)
	for k, v := range m.activeMonitors {
		result[k] = v
	}
	return result
}
