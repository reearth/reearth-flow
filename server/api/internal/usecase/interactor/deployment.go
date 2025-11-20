package interactor

import (
	"context"
	"fmt"
	"net/url"
	"strconv"
	"strings"
	"time"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/usecasex"
)

type Deployment struct {
	deploymentRepo    repo.Deployment
	projectRepo       repo.Project
	workflowRepo      repo.Workflow
	jobRepo           repo.Job
	workerConfigRepo  repo.WorkerConfig
	triggerRepo       repo.Trigger
	transaction       usecasex.Transaction
	batch             gateway.Batch
	file              gateway.File
	job               interfaces.Job
	permissionChecker gateway.PermissionChecker
}

func NewDeployment(r *repo.Container, gr *gateway.Container, jobUsecase interfaces.Job, permissionChecker gateway.PermissionChecker) interfaces.Deployment {
	return &Deployment{
		deploymentRepo:    r.Deployment,
		projectRepo:       r.Project,
		workflowRepo:      r.Workflow,
		jobRepo:           r.Job,
		workerConfigRepo:  r.WorkerConfig,
		triggerRepo:       r.Trigger,
		transaction:       r.Transaction,
		batch:             gr.Batch,
		file:              gr.File,
		job:               jobUsecase,
		permissionChecker: permissionChecker,
	}
}

func (i *Deployment) checkPermission(ctx context.Context, action string) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceDeployment, action)
}

func (i *Deployment) Fetch(ctx context.Context, ids []id.DeploymentID) ([]*deployment.Deployment, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	return i.deploymentRepo.FindByIDs(ctx, ids)
}

func (i *Deployment) FindByWorkspace(ctx context.Context, id id.WorkspaceID, p *interfaces.PaginationParam, keyword *string) ([]*deployment.Deployment, *interfaces.PageBasedInfo, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, nil, err
	}

	return i.deploymentRepo.FindByWorkspace(ctx, id, p, keyword)
}

func (i *Deployment) FindByProject(ctx context.Context, id id.ProjectID) (*deployment.Deployment, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	return i.deploymentRepo.FindByProject(ctx, id)
}

func (i *Deployment) FindByVersion(ctx context.Context, wsID id.WorkspaceID, projectID *id.ProjectID, version string) (*deployment.Deployment, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	return i.deploymentRepo.FindByVersion(ctx, wsID, projectID, version)
}

func (i *Deployment) FindHead(ctx context.Context, wsID id.WorkspaceID, projectID *id.ProjectID) (*deployment.Deployment, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	return i.deploymentRepo.FindHead(ctx, wsID, projectID)
}

func (i *Deployment) FindVersions(ctx context.Context, wsID id.WorkspaceID, projectID *id.ProjectID) ([]*deployment.Deployment, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	return i.deploymentRepo.FindVersions(ctx, wsID, projectID)
}

func incrementVersion(version string) string {
	if strings.HasPrefix(version, "v") {
		currentVersion, err := strconv.Atoi(version[1:])
		if err == nil {
			return fmt.Sprintf("v%d", currentVersion+1)
		}
	}
	return "v1"
}

func (i *Deployment) Create(ctx context.Context, dp interfaces.CreateDeploymentParam) (result *deployment.Deployment, err error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return
	}

	ctx = tx.Context()
	defer func() {
		if err2 := tx.End(ctx); err == nil && err2 != nil {
			err = err2
		}
	}()

	if dp.Project != nil {
		_, err = i.projectRepo.FindByID(ctx, *dp.Project)
		if err != nil {
			return nil, err
		}
	}

	url, err := i.file.UploadWorkflow(ctx, dp.Workflow)
	if err != nil {
		return nil, err
	}

	d := deployment.New().
		NewID().
		Description(dp.Description).
		Workspace(dp.Workspace).
		WorkflowURL(url.String())

	if dp.Project != nil {
		d = d.Project(dp.Project)

		head, _ := i.deploymentRepo.FindHead(ctx, dp.Workspace, dp.Project)

		d = d.IsHead(true)
		if head != nil {
			currentHeadID := head.ID()
			d = d.HeadID(&currentHeadID)
			d = d.Version(incrementVersion(head.Version()))

			head.SetIsHead(false)
			if err := i.deploymentRepo.Save(ctx, head); err != nil {
				return nil, err
			}
		} else {
			d = d.Version("v1")
		}
	} else {
		d = d.Version("v0")
		d = d.IsHead(false)
	}

	dep, err := d.Build()
	if err != nil {
		return nil, err
	}

	if err := i.deploymentRepo.Save(ctx, dep); err != nil {
		return nil, err
	}

	tx.Commit()
	return dep, nil
}

func (i *Deployment) Update(ctx context.Context, dp interfaces.UpdateDeploymentParam) (_ *deployment.Deployment, err error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return
	}

	ctx = tx.Context()
	defer func() {
		if err2 := tx.End(ctx); err == nil && err2 != nil {
			err = err2
		}
	}()

	d, err := i.deploymentRepo.FindByID(ctx, dp.ID)
	if err != nil {
		return nil, err
	}

	if dp.Workflow != nil {
		if url, _ := url.Parse(d.WorkflowURL()); url != nil {
			if err := i.file.RemoveWorkflow(ctx, url); err != nil {
				return nil, err
			}
		}

		url, err := i.file.UploadWorkflow(ctx, dp.Workflow)
		if err != nil {
			return nil, err
		}
		d.SetWorkflowURL(url.String())

		if d.Project() != nil {
			currentHead, err := i.deploymentRepo.FindHead(ctx, d.Workspace(), d.Project())
			if err != nil {
				return nil, err
			}

			d.SetVersion(incrementVersion(currentHead.Version()))
			d.SetIsHead(true)
			if currentHead != nil && currentHead.ID() != d.ID() {
				d.SetHeadID(currentHead.ID())
				currentHead.SetIsHead(false)
				if err := i.deploymentRepo.Save(ctx, currentHead); err != nil {
					return nil, err
				}
			}
		}
	}

	if dp.Description != nil {
		d.SetDescription(*dp.Description)
	}

	if err := i.deploymentRepo.Save(ctx, d); err != nil {
		return nil, err
	}

	tx.Commit()
	return d, nil
}

func (i *Deployment) Delete(ctx context.Context, deploymentID id.DeploymentID) (err error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return err
	}

	triggers, err := i.triggerRepo.FindByDeployment(ctx, deploymentID)
	if err != nil {
		return err
	}
	if len(triggers) > 0 {
		return interfaces.ErrDeploymentHasTriggers
	}

	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return
	}

	ctx = tx.Context()
	defer func() {
		if err2 := tx.End(ctx); err == nil && err2 != nil {
			err = err2
		}
	}()

	dep, err := i.deploymentRepo.FindByID(ctx, deploymentID)
	if err != nil {
		return err
	}

	if dep.Project() != nil {
		versions, err := i.deploymentRepo.FindVersions(ctx, dep.Workspace(), dep.Project())
		if err != nil {
			return err
		}

		for _, version := range versions {
			if url, _ := url.Parse(version.WorkflowURL()); url != nil {
				if err := i.file.RemoveWorkflow(ctx, url); err != nil {
					return err
				}
			}

			if err := i.deploymentRepo.Remove(ctx, version.ID()); err != nil {
				return err
			}
		}
	} else {
		if url, _ := url.Parse(dep.WorkflowURL()); url != nil {
			if err := i.file.RemoveWorkflow(ctx, url); err != nil {
				return err
			}
		}

		if err := i.deploymentRepo.Remove(ctx, deploymentID); err != nil {
			return err
		}
	}

	tx.Commit()
	return nil
}

func (i *Deployment) Execute(ctx context.Context, p interfaces.ExecuteDeploymentParam) (_ *job.Job, err error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return
	}

	ctx = tx.Context()
	defer func() {
		if err2 := tx.End(ctx); err == nil && err2 != nil {
			err = err2
		}
	}()

	d, err := i.deploymentRepo.FindByID(ctx, p.DeploymentID)
	if err != nil {
		return nil, err
	}

	debug := false

	j, err := job.New().
		NewID().
		Debug(&debug).
		Deployment(d.ID()).
		Workspace(d.Workspace()).
		Status(job.StatusPending).
		StartedAt(time.Now()).
		Build()
	if err != nil {
		return nil, err
	}

	metadataURL, err := i.file.UploadMetadata(ctx, j.ID().String(), []string{}) // TODO: add assets
	if err != nil {
		return nil, fmt.Errorf("failed to upload metadata: %v", err)
	}
	if metadataURL != nil {
		j.SetMetadataURL(metadataURL.String())
	}

	if err := i.jobRepo.Save(ctx, j); err != nil {
		return nil, err
	}

	var projectID id.ProjectID
	if d.Project() != nil {
		projectID = *d.Project()
	}

	gcpJobID, err := i.batch.SubmitJob(ctx, j.ID(), d.WorkflowURL(), j.MetadataURL(), nil, projectID, d.Workspace())
	if err != nil {
		return nil, interfaces.ErrJobCreationFailed
	}
	j.SetGCPJobID(gcpJobID)

	if err := i.jobRepo.Save(ctx, j); err != nil {
		return nil, err
	}

	tx.Commit()

	if err := i.job.StartMonitoring(ctx, j, nil); err != nil {
		return nil, fmt.Errorf("failed to start job monitoring: %v", err)
	}

	return j, nil
}
