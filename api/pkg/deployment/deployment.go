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
	updatedAt   time.Time
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
	return d.id.Timestamp()
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

func (d *Deployment) SetUpdatedAt(updatedAt time.Time) {
	d.updatedAt = updatedAt
}
