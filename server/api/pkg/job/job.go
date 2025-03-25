package job

import (
	"time"
)

type Status string

const (
	StatusPending   Status = "PENDING"
	StatusRunning   Status = "RUNNING"
	StatusCancelled Status = "CANCELLED"
	StatusCompleted Status = "COMPLETED"
	StatusFailed    Status = "FAILED"
)

type Job struct {
	completedAt *time.Time
	debug       *bool
	deployment  DeploymentID
	gcpJobID    string
	id          ID
	logsURL     string
	metadataURL string
	outputURLs  []string
	startedAt   time.Time
	status      Status
	workspace   WorkspaceID
}

func NewJob(id ID, deployment DeploymentID, workspace WorkspaceID, gcpJobID string) *Job {
	return &Job{
		deployment:  deployment,
		gcpJobID:    gcpJobID,
		id:          id,
		metadataURL: "",
		status:      StatusPending,
		startedAt:   time.Now(),
		workspace:   workspace,
	}
}

func (j *Job) ID() ID {
	return j.id
}

func (j *Job) Debug() *bool {
	return j.debug
}

func (j *Job) Deployment() DeploymentID {
	return j.deployment
}

func (j *Job) Workspace() WorkspaceID {
	return j.workspace
}

func (j *Job) GCPJobID() string {
	return j.gcpJobID
}

func (j *Job) Status() Status {
	return j.status
}

func (j *Job) StartedAt() time.Time {
	return j.startedAt
}

func (j *Job) CompletedAt() *time.Time {
	return j.completedAt
}

func (j *Job) LogsURL() string {
	return j.logsURL
}

func (j *Job) MetadataURL() string {
	return j.metadataURL
}

func (j *Job) OutputURLs() []string {
	return j.outputURLs
}

func (j *Job) SetID(id ID) {
	j.id = id
}

func (j *Job) SetDebug(debug *bool) {
	j.debug = debug
}

func (j *Job) SetDeployment(deployment DeploymentID) {
	j.deployment = deployment
}

func (j *Job) SetWorkspace(workspace WorkspaceID) {
	j.workspace = workspace
}

func (j *Job) SetGCPJobID(gcpJobID string) {
	j.gcpJobID = gcpJobID
}

func (j *Job) SetStatus(status Status) {
	j.status = status
	if status == StatusCompleted || status == StatusFailed {
		now := time.Now()
		j.completedAt = &now
	}
}

func (j *Job) SetStartedAt(startedAt time.Time) {
	j.startedAt = startedAt
}

func (j *Job) SetCompletedAt(completedAt *time.Time) {
	j.completedAt = completedAt
}

func (j *Job) SetLogsURL(logsURL string) {
	j.logsURL = logsURL
}

func (j *Job) SetMetadataURL(metadataURL string) {
	j.metadataURL = metadataURL
}

func (j *Job) SetOutputURLs(outputURLs []string) {
	j.outputURLs = outputURLs
}
