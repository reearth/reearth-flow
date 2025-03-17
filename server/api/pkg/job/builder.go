package job

import (
	"time"
)

type JobBuilder struct {
	j *Job
}

func New() *JobBuilder {
	return &JobBuilder{j: &Job{}}
}

func (b *JobBuilder) Build() (*Job, error) {
	if b.j.id.IsNil() {
		return nil, ErrInvalidID
	}
	return b.j, nil
}

func (b *JobBuilder) MustBuild() *Job {
	r, err := b.Build()
	if err != nil {
		panic(err)
	}
	return r
}

func (b *JobBuilder) ID(id ID) *JobBuilder {
	b.j.id = id
	return b
}

func (b *JobBuilder) NewID() *JobBuilder {
	b.j.id = NewID()
	return b
}

func (b *JobBuilder) Debug(debug *bool) *JobBuilder {
	b.j.debug = debug
	return b
}

func (b *JobBuilder) Deployment(deployment DeploymentID) *JobBuilder {
	b.j.deployment = deployment
	return b
}

func (b *JobBuilder) Workspace(workspace WorkspaceID) *JobBuilder {
	b.j.workspace = workspace
	return b
}

func (b *JobBuilder) GCPJobID(gcpJobID string) *JobBuilder {
	b.j.gcpJobID = gcpJobID
	return b
}

func (b *JobBuilder) Status(status Status) *JobBuilder {
	b.j.status = status
	return b
}

func (b *JobBuilder) StartedAt(startedAt time.Time) *JobBuilder {
	b.j.startedAt = startedAt
	return b
}

func (b *JobBuilder) CompletedAt(completedAt *time.Time) *JobBuilder {
	b.j.completedAt = completedAt
	return b
}

func (b *JobBuilder) MetadataURL(metadataURL string) *JobBuilder {
	b.j.metadataURL = metadataURL
	return b
}

func (b *JobBuilder) OutputURLs(outputURLs []string) *JobBuilder {
	b.j.outputURLs = outputURLs
	return b
}

func (b *JobBuilder) LogsURL(logsURL string) *JobBuilder {
	b.j.logsURL = logsURL
	return b
}
