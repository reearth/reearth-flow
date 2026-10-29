package deployment

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/parameter"
)

type Deployment struct {
	updatedAt   time.Time
	project     *ProjectID
	headId      *ID
	workflowURL string
	description string
	version     string
	// parameters is the project's workflow variables as they stood when this
	// version was deployed, so triggers and runs read the definitions that
	// match the workflow file rather than the project's current ones.
	parameters []*parameter.Parameter
	id         ID
	workspace  WorkspaceID
	isHead     bool
}

func (d *Deployment) ID() ID {
	return d.id
}

func (d *Deployment) Project() *ProjectID {
	return d.project
}

func (d *Deployment) Workspace() WorkspaceID {
	return d.workspace
}

func (d *Deployment) WorkflowURL() string {
	return d.workflowURL
}

func (d *Deployment) Description() string {
	return d.description
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

func (d *Deployment) HeadID() *ID {
	return d.headId
}

func (d *Deployment) IsHead() bool {
	return d.isHead
}

func (d *Deployment) Parameters() []*parameter.Parameter {
	return d.parameters
}

func (d *Deployment) SetID(id ID) {
	d.id = id
}

func (d *Deployment) SetProject(project *ProjectID) {
	d.project = project
	d.updatedAt = time.Now()
}

func (d *Deployment) SetWorkspace(workspace WorkspaceID) {
	d.workspace = workspace
	d.updatedAt = time.Now()
}

func (d *Deployment) SetWorkflowURL(workflowURL string) {
	d.workflowURL = workflowURL
	d.updatedAt = time.Now()
}

func (d *Deployment) SetDescription(description string) {
	d.description = description
	d.updatedAt = time.Now()
}

func (d *Deployment) SetVersion(version string) {
	d.version = version
	d.updatedAt = time.Now()
}

func (d *Deployment) SetHeadID(headId ID) {
	d.headId = &headId
	d.updatedAt = time.Now()
}

func (d *Deployment) SetIsHead(isHead bool) {
	d.isHead = isHead
	d.updatedAt = time.Now()
}

func (d *Deployment) SetParameters(parameters []*parameter.Parameter) {
	d.parameters = parameters
	d.updatedAt = time.Now()
}
