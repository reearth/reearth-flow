package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"golang.org/x/exp/slices"
)

type JobDocument struct {
	ID                string     `bson:"id"`
	Debug             *bool      `bson:"debug"`
	DeploymentID      string     `bson:"deploymentid"`
	WorkspaceID       string     `bson:"workspaceid"`
	GCPJobID          string     `bson:"gcpjobid"`
	LogsURL           string     `bson:"logsurl"`
	WorkerLogsURL     string     `bson:"workerlogsurl"`
	UserFacingLogsURL string     `bson:"userfacinglogsurl"`
	Status            string     `bson:"status"`
	BatchStatus       *string    `bson:"batchstatus,omitempty"`
	WorkerStatus      *string    `bson:"workerstatus,omitempty"`
	StartedAt         time.Time  `bson:"startedat"`
	CompletedAt       *time.Time `bson:"completedat"`
	MetadataURL       string     `bson:"metadataurl"`
	OutputURLs        []string   `bson:"outputurls"`
}

type JobConsumer = Consumer[*JobDocument, *job.Job]

func NewJobConsumer(workspaces []id.WorkspaceID) *JobConsumer {
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

	var batchStatus *string
	if j.BatchStatus() != nil {
		s := string(*j.BatchStatus())
		batchStatus = &s
	}

	var workerStatus *string
	if j.WorkerStatus() != nil {
		s := string(*j.WorkerStatus())
		workerStatus = &s
	}

	doc := &JobDocument{
		ID:                jid,
		Debug:             j.Debug(),
		DeploymentID:      j.Deployment().String(),
		WorkspaceID:       j.Workspace().String(),
		GCPJobID:          j.GCPJobID(),
		LogsURL:           j.LogsURL(),
		WorkerLogsURL:     j.WorkerLogsURL(),
		UserFacingLogsURL: j.UserFacingLogsURL(),
		Status:            string(j.Status()),
		BatchStatus:       batchStatus,
		WorkerStatus:      workerStatus,
		StartedAt:         j.StartedAt(),
		CompletedAt:       j.CompletedAt(),
		MetadataURL:       j.MetadataURL(),
		OutputURLs:        j.OutputURLs(),
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

	wid, err := id.WorkspaceIDFrom(d.WorkspaceID)
	if err != nil {
		return nil, err
	}

	var batchStatus *job.Status
	if d.BatchStatus != nil {
		s := job.Status(*d.BatchStatus)
		batchStatus = &s
	}

	var workerStatus *job.Status
	if d.WorkerStatus != nil {
		s := job.Status(*d.WorkerStatus)
		workerStatus = &s
	}

	j := job.New().
		ID(jid).
		Debug(d.Debug).
		Deployment(did).
		Workspace(wid).
		Status(job.Status(d.Status)).
		BatchStatus(batchStatus).
		WorkerStatus(workerStatus).
		StartedAt(d.StartedAt).
		MetadataURL(d.MetadataURL).
		GCPJobID(d.GCPJobID).
		OutputURLs(d.OutputURLs).
		LogsURL(d.LogsURL).
		WorkerLogsURL(d.WorkerLogsURL).
		UserFacingLogsURL(d.UserFacingLogsURL)

	if d.CompletedAt != nil {
		j = j.CompletedAt(d.CompletedAt)
	}

	jobModel, err := j.Build()
	if err != nil {
		return nil, err
	}

	return jobModel, nil
}
