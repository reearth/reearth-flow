package job

import "time"

type Status string

const (
	StatusPending   Status = "PENDING"
	StatusRunning   Status = "RUNNING"
	StatusCompleted Status = "COMPLETED"
	StatusFailed    Status = "FAILED"
)

type Job struct {
	id          ID
	deployment  DeploymentID
	workspace   WorkspaceID
	gcpJobID    string
	status      Status
	startedAt   time.Time
	completedAt *time.Time
}

func NewJob(id ID, deployment DeploymentID, workspace WorkspaceID, gcpJobID string) *Job {
	return &Job{
		id:         id,
		deployment: deployment,
		workspace:  workspace,
		gcpJobID:   gcpJobID,
		status:     StatusPending,
		startedAt:  time.Now(),
	}
}

func (j *Job) ID() ID {
	return j.id
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

func (j *Job) SetID(id ID) {
	j.id = id
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
