package deployment

import (
	"time"
)

type Deployment struct {
	id          ID
	project     ProjectID
	workspace   WorkspaceID
	workflowUrl string
	version     string
	createdAt   time.Time
	updatedAt   time.Time
}

func NewDeployment(id ID, project ProjectID, workspace WorkspaceID, workflowUrl string, version string) *Deployment {
	now := time.Now()
	return &Deployment{
		id:          id,
		project:     project,
		workspace:   workspace,
		workflowUrl: workflowUrl,
		version:     version,
		createdAt:   now,
		updatedAt:   now,
	}
}

func (d *Deployment) ID() ID {
	return d.id
}

func (d *Deployment) Project() ProjectID {
	return d.project
}

func (d *Deployment) Workspace() WorkspaceID {
	return d.workspace
}

func (d *Deployment) WorkflowUrl() string {
	return d.workflowUrl
}

func (d *Deployment) Version() string {
	return d.version
}

func (d *Deployment) CreatedAt() time.Time {
	return d.createdAt
}

func (d *Deployment) UpdatedAt() time.Time {
	return d.updatedAt
}

func (d *Deployment) SetID(id ID) {
	d.id = id
}

func (d *Deployment) SetProject(project ProjectID) {
	d.project = project
	d.updatedAt = time.Now()
}

func (d *Deployment) SetWorkspace(workspace WorkspaceID) {
	d.workspace = workspace
	d.updatedAt = time.Now()
}

func (d *Deployment) SetWorkflowUrl(workflowUrl string) {
	d.workflowUrl = workflowUrl
	d.updatedAt = time.Now()
}

func (d *Deployment) SetVersion(version string) {
	d.version = version
	d.updatedAt = time.Now()
}
