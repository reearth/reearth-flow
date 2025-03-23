package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/account/accountdomain"
	"golang.org/x/exp/slices"
)

type JobDocument struct {
	ID            string     `bson:"id"`
	Debug         *bool      `bson:"debug"`
	DeploymentID  string     `bson:"deploymentid"`
	WorkspaceID   string     `bson:"workspaceid"`
	GCPJobID      string     `bson:"gcpjobid"`
	LogsURL       string     `bson:"logsurl"`
	WorkerLogsURL string     `bson:"workerlogsurl"`
	Status        string     `bson:"status"`
	StartedAt     time.Time  `bson:"startedat"`
	CompletedAt   *time.Time `bson:"completedat"`
	MetadataURL   string     `bson:"metadataurl"`
	OutputURLs    []string   `bson:"outputurls"`
}

type JobConsumer = Consumer[*JobDocument, *job.Job]

func NewJobConsumer(workspaces []accountdomain.WorkspaceID) *JobConsumer {
	return NewConsumer[*JobDocument](func(j *job.Job) bool {
		result := workspaces == nil || slices.Contains(workspaces, j.Workspace())
		return result
	})
}

func NewJob(j *job.Job) (*JobDocument, string) {
	if j == nil {
		return nil, ""
	}

	jid := j.ID().String()

	doc := &JobDocument{
		ID:            jid,
		Debug:         j.Debug(),
		DeploymentID:  j.Deployment().String(),
		WorkspaceID:   j.Workspace().String(),
		GCPJobID:      j.GCPJobID(),
		LogsURL:       j.LogsURL(),
		WorkerLogsURL: j.WorkerLogsURL(),
		Status:        string(j.Status()),
		StartedAt:     j.StartedAt(),
		CompletedAt:   j.CompletedAt(),
		MetadataURL:   j.MetadataURL(),
		OutputURLs:    j.OutputURLs(),
	}

	return doc, jid
}

func (d *JobDocument) Model() (*job.Job, error) {
	if d == nil {
		return nil, nil
	}

	jid, err := id.JobIDFrom(d.ID)
	if err != nil {
		return nil, err
	}

	did, err := id.DeploymentIDFrom(d.DeploymentID)
	if err != nil {
		return nil, err
	}

	wid, err := accountdomain.WorkspaceIDFrom(d.WorkspaceID)
	if err != nil {
		return nil, err
	}

	j := job.New().
		ID(jid).
		Debug(d.Debug).
		Deployment(did).
		Workspace(wid).
		Status(job.Status(d.Status)).
		StartedAt(d.StartedAt).
		MetadataURL(d.MetadataURL).
		GCPJobID(d.GCPJobID).
		OutputURLs(d.OutputURLs).
		LogsURL(d.LogsURL).
		WorkerLogsURL(d.WorkerLogsURL)

	if d.CompletedAt != nil {
		j = j.CompletedAt(d.CompletedAt)
	}

	jobModel, err := j.Build()
	if err != nil {
		return nil, err
	}

	return jobModel, nil
}
