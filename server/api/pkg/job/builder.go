package job

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/variable"
)

type Builder struct {
	j *Job
}

func New() *Builder {
	return &Builder{j: &Job{}}
}

func (b *Builder) Build() (*Job, error) {
	if b.j.id.IsNil() {
		return nil, ErrInvalidID
	}
	return b.j, nil
}

func (b *Builder) MustBuild() *Job {
	r, err := b.Build()
	if err != nil {
		panic(err)
	}
	return r
}

func (b *Builder) BatchStatus(batchStatus *Status) *Builder {
	b.j.batchStatus = batchStatus
	return b
}

func (b *Builder) ID(id ID) *Builder {
	b.j.id = id
	return b
}

func (b *Builder) NewID() *Builder {
	b.j.id = NewID()
	return b
}

func (b *Builder) Debug(debug *bool) *Builder {
	b.j.debug = debug
	return b
}

func (b *Builder) Deployment(deployment DeploymentID) *Builder {
	b.j.deployment = deployment
	return b
}

func (b *Builder) Workspace(workspace WorkspaceID) *Builder {
	b.j.workspace = workspace
	return b
}

func (b *Builder) GCPJobID(gcpJobID string) *Builder {
	b.j.gcpJobID = gcpJobID
	return b
}

func (b *Builder) Status(status Status) *Builder {
	b.j.status = status
	return b
}

func (b *Builder) StartedAt(startedAt time.Time) *Builder {
	b.j.startedAt = startedAt
	return b
}

func (b *Builder) CompletedAt(completedAt *time.Time) *Builder {
	b.j.completedAt = completedAt
	return b
}

func (b *Builder) MetadataURL(metadataURL string) *Builder {
	b.j.metadataURL = metadataURL
	return b
}

func (b *Builder) OutputURLs(outputURLs []string) *Builder {
	b.j.outputURLs = outputURLs
	return b
}

func (b *Builder) LogsURL(logsURL string) *Builder {
	b.j.logsURL = logsURL
	return b
}

func (b *Builder) WorkerLogsURL(workerLogsURL string) *Builder {
	b.j.workerLogsURL = workerLogsURL
	return b
}

func (b *Builder) UserFacingLogsURL(userFacingLogsURL string) *Builder {
	b.j.userFacingLogsURL = userFacingLogsURL
	return b
}

func (b *Builder) WorkerStatus(workerStatus *Status) *Builder {
	b.j.workerStatus = workerStatus
	return b
}

func (b *Builder) Variables(variables []variable.Variable) *Builder {
	b.j.variables = variables
	return b
}
