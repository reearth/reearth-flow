package mongodoc

import (
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"golang.org/x/exp/slices"
)

type JobDocument struct {
	StartedAt         time.Time           `bson:"startedat"`
	Debug             *bool               `bson:"debug"`
	BatchStatus       *string             `bson:"batchstatus,omitempty"`
	WorkerStatus      *string             `bson:"workerstatus,omitempty"`
	CompletedAt       *time.Time          `bson:"completedat"`
	Parameters        []ParameterDocument `bson:"parameters,omitempty"`
	ID                string              `bson:"id"`
	DeploymentID      *string             `bson:"deploymentid,omitempty"`
	WorkspaceID       string              `bson:"workspaceid"`
	ProjectID         *string             `bson:"projectid,omitempty"`
	ProjectVersion    *int                `bson:"projectversion,omitempty"`
	GCPJobID          string              `bson:"gcpjobid"`
	LogsURL           string              `bson:"logsurl"`
	WorkerLogsURL     string              `bson:"workerlogsurl"`
	UserFacingLogsURL string              `bson:"userfacinglogsurl"`
	Status            string              `bson:"status"`
	MetadataURL       string              `bson:"metadataurl"`
	OutputURLs        []string            `bson:"outputurls"`
}

type JobConsumer = Consumer[*JobDocument, *job.Job]

func NewJobConsumer(workspaces []accountsid.WorkspaceID) *JobConsumer {
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

	var deploymentID *string
	if j.Deployment() != nil {
		s := j.Deployment().String()
		deploymentID = &s
	}

	var projectID *string
	if j.ProjectID() != nil {
		s := j.ProjectID().String()
		projectID = &s
	}

	doc := &JobDocument{
		ID:                jid,
		Debug:             j.Debug(),
		DeploymentID:      deploymentID,
		WorkspaceID:       j.Workspace().String(),
		ProjectID:         projectID,
		ProjectVersion:    j.ProjectVersion(),
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

	if p := j.Parameters(); len(p) > 0 {
		// IDs not needed; parameters are embedded in the job document
		doc.Parameters, _ = ParametersToDocs(p)
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

	var did *id.DeploymentID
	if d.DeploymentID != nil {
		deploymentID, err := id.DeploymentIDFrom(*d.DeploymentID)
		if err != nil {
			return nil, err
		}
		did = &deploymentID
	}

	wid, err := accountsid.WorkspaceIDFrom(d.WorkspaceID)
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

	if ps := ParametersFromDocs(d.Parameters); len(ps) > 0 {
		j = j.Parameters(ps)
	}

	jobModel, err := j.Build()
	if err != nil {
		return nil, err
	}

	return jobModel, nil
}
