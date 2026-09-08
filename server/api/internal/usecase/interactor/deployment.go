package interactor

import (
	"context"
	"errors"
	"fmt"
	"net/url"
	"strconv"
	"strings"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
)

type Deployment struct {
	deploymentRepo    repo.Deployment
	projectRepo       repo.Project
	workflowRepo      repo.Workflow
	jobRepo           repo.Job
	workerConfigRepo  repo.WorkerConfig
	triggerRepo       repo.Trigger
	transaction       usecasex.Transactor
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

func (i *Deployment) checkPermission(ctx context.Context, action string, workspaceID ...accountsid.WorkspaceID) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceDeployment, action, workspaceID...)
}

func (i *Deployment) Fetch(ctx context.Context, ids []id.DeploymentID) ([]*deployment.Deployment, error) {
	deployments, err := i.deploymentRepo.FindByIDs(ctx, ids)
	if err != nil {
		return nil, err
	}

	// FindByIDs pads not-found/unreadable entries with nil, so the first
	// element isn't necessarily a deployment — use the first non-nil one.
	var ws accountsid.WorkspaceID
	var haveWorkspace bool
	for _, d := range deployments {
		if d != nil {
			ws, haveWorkspace = d.Workspace(), true
			break
		}
	}

	if !haveWorkspace {
		if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
			return nil, err
		}
	} else {
		if err := i.checkPermission(ctx, rbac.ActionAny, ws); err != nil { // single-workspace batch assumption
			return nil, err
		}
	}

	return deployments, nil
}

func (i *Deployment) FindByWorkspace(ctx context.Context, id accountsid.WorkspaceID, p *interfaces.PaginationParam, keyword *string) ([]*deployment.Deployment, *interfaces.PageBasedInfo, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny, id); err != nil {
		return nil, nil, err
	}

	return i.deploymentRepo.FindByWorkspace(ctx, id, p, keyword)
}

func (i *Deployment) FindByProject(ctx context.Context, id id.ProjectID) (*deployment.Deployment, error) {
	project, err := i.projectRepo.FindByID(ctx, id)
	if err != nil {
		return nil, err
	}
	if project == nil {
		return nil, fmt.Errorf("project not found: %s", id)
	}
	if err := i.checkPermission(ctx, rbac.ActionAny, project.Workspace()); err != nil {
		return nil, err
	}

	return i.deploymentRepo.FindByProject(ctx, id)
}

// FindByProjects batches FindByProject for a dataloader: one project lookup and
// one permission check per distinct workspace instead of one of each per project.
// Projects in a workspace the caller can't see are simply omitted from the result.
func (i *Deployment) FindByProjects(ctx context.Context, ids []id.ProjectID) (map[id.ProjectID]*deployment.Deployment, error) {
	if len(ids) == 0 {
		if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
			return nil, err
		}
		return map[id.ProjectID]*deployment.Deployment{}, nil
	}

	projects, err := i.projectRepo.FindByIDs(ctx, ids)
	if err != nil {
		return nil, err
	}

	byWorkspace := map[accountsid.WorkspaceID][]id.ProjectID{}
	for _, p := range projects {
		if p == nil { // some repo implementations pad not-found/unreadable entries with nil
			continue
		}
		byWorkspace[p.Workspace()] = append(byWorkspace[p.Workspace()], p.ID())
	}

	result := make(map[id.ProjectID]*deployment.Deployment, len(projects))
	for ws, pids := range byWorkspace {
		if err := i.checkPermission(ctx, rbac.ActionAny, ws); err != nil {
			continue // caller can't see this workspace; omit its projects' deployments
		}
		for _, pid := range pids {
			dep, err := i.deploymentRepo.FindByProject(ctx, pid)
			if err != nil {
				if errors.Is(err, rerror.ErrNotFound) {
					continue // project has no deployment yet
				}
				return nil, err
			}
			if dep != nil {
				result[pid] = dep
			}
		}
	}

	return result, nil
}

func (i *Deployment) FindByVersion(ctx context.Context, wsID accountsid.WorkspaceID, projectID *id.ProjectID, version string) (*deployment.Deployment, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny, wsID); err != nil {
		return nil, err
	}

	return i.deploymentRepo.FindByVersion(ctx, wsID, projectID, version)
}

func (i *Deployment) FindHead(ctx context.Context, wsID accountsid.WorkspaceID, projectID *id.ProjectID) (*deployment.Deployment, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny, wsID); err != nil {
		return nil, err
	}

	return i.deploymentRepo.FindHead(ctx, wsID, projectID)
}

func (i *Deployment) FindVersions(ctx context.Context, wsID accountsid.WorkspaceID, projectID *id.ProjectID) ([]*deployment.Deployment, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny, wsID); err != nil {
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

func (i *Deployment) Create(ctx context.Context, dp interfaces.CreateDeploymentParam) (*deployment.Deployment, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny, dp.Workspace); err != nil {
		return nil, err
	}

	var result *deployment.Deployment
	if err := i.transaction.WithinTransaction(ctx, func(ctx context.Context) error {
		if dp.Project != nil {
			if _, err := i.projectRepo.FindByID(ctx, *dp.Project); err != nil {
				return err
			}
		}

		url, err := i.file.UploadWorkflow(ctx, dp.Workflow)
		if err != nil {
			return err
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
					return err
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
			return err
		}

		if err := i.deploymentRepo.Save(ctx, dep); err != nil {
			return err
		}

		result = dep
		return nil
	}); err != nil {
		return nil, err
	}
	return result, nil
}

func (i *Deployment) Update(ctx context.Context, dp interfaces.UpdateDeploymentParam) (*deployment.Deployment, error) {
	dep, err := i.deploymentRepo.FindByID(ctx, dp.ID)
	if err != nil {
		return nil, err
	}
	if dep == nil {
		return nil, fmt.Errorf("deployment not found: %s", dp.ID)
	}
	if err := i.checkPermission(ctx, rbac.ActionAny, dep.Workspace()); err != nil {
		return nil, err
	}

	var (
		result              *deployment.Deployment
		previousWorkflowURL string
	)
	if err := i.transaction.WithinTransaction(ctx, func(ctx context.Context) error {
		d, err := i.deploymentRepo.FindByID(ctx, dp.ID)
		if err != nil {
			return err
		}
		if d == nil {
			return fmt.Errorf("deployment not found: %s", dp.ID)
		}

		if dp.Workflow != nil {
			// Captured, not deleted, until the transaction commits: removing it here
			// is irreversible, so any later failure (or a serialization retry) would
			// roll the row back to a WorkflowURL whose object no longer exists.
			previousWorkflowURL = d.WorkflowURL()

			u, err := i.file.UploadWorkflow(ctx, dp.Workflow)
			if err != nil {
				return err
			}
			d.SetWorkflowURL(u.String())

			if d.Project() != nil {
				currentHead, err := i.deploymentRepo.FindHead(ctx, d.Workspace(), d.Project())
				if err != nil {
					return err
				}

				// Defensive: every repo returns ErrNotFound rather than a nil head,
				// so this cannot be nil today. incrementVersion("") is "v1", which
				// is what Create uses for a project with no head.
				var headVersion string
				if currentHead != nil {
					headVersion = currentHead.Version()
				}
				d.SetVersion(incrementVersion(headVersion))
				d.SetIsHead(true)
				if currentHead != nil && currentHead.ID() != d.ID() {
					d.SetHeadID(currentHead.ID())
					currentHead.SetIsHead(false)
					if err := i.deploymentRepo.Save(ctx, currentHead); err != nil {
						return err
					}
				}
			}
		}

		if dp.Description != nil {
			d.SetDescription(*dp.Description)
		}

		if err := i.deploymentRepo.Save(ctx, d); err != nil {
			return err
		}

		result = d
		return nil
	}); err != nil {
		return nil, err
	}

	// Only now is it safe to drop the old object: the row that stopped referencing
	// it is durable. Best-effort, since a failure here leaks an object rather than
	// breaking the deployment.
	if previousWorkflowURL != "" && previousWorkflowURL != result.WorkflowURL() {
		if u, _ := url.Parse(previousWorkflowURL); u != nil {
			if err := i.file.RemoveWorkflow(ctx, u); err != nil {
				log.Errorfc(ctx, "deployment: could not remove superseded workflow %s: %v", previousWorkflowURL, err)
			}
		}
	}

	return result, nil
}

func (i *Deployment) Delete(ctx context.Context, deploymentID id.DeploymentID) (err error) {
	d, err := i.deploymentRepo.FindByID(ctx, deploymentID)
	if err != nil {
		return err
	}
	if d == nil {
		return fmt.Errorf("deployment not found: %s", deploymentID)
	}
	if err := i.checkPermission(ctx, rbac.ActionAny, d.Workspace()); err != nil {
		return err
	}

	triggers, err := i.triggerRepo.FindByDeployment(ctx, deploymentID)
	if err != nil {
		return err
	}
	if len(triggers) > 0 {
		return interfaces.ErrDeploymentHasTriggers
	}

	var orphanedWorkflows []string
	if err := i.transaction.WithinTransaction(ctx, func(ctx context.Context) error {
		dep, err := i.deploymentRepo.FindByID(ctx, deploymentID)
		if err != nil {
			return err
		}
		if dep == nil {
			return fmt.Errorf("deployment not found: %s", deploymentID)
		}

		// Reset so a retried attempt does not accumulate the previous one's entries.
		orphanedWorkflows = nil

		if dep.Project() != nil {
			versions, err := i.deploymentRepo.FindVersions(ctx, dep.Workspace(), dep.Project())
			if err != nil {
				return err
			}

			for _, version := range versions {
				orphanedWorkflows = append(orphanedWorkflows, version.WorkflowURL())

				if err := i.deploymentRepo.Remove(ctx, version.ID()); err != nil {
					return err
				}
			}
		} else {
			orphanedWorkflows = append(orphanedWorkflows, dep.WorkflowURL())

			if err := i.deploymentRepo.Remove(ctx, deploymentID); err != nil {
				return err
			}
		}

		return nil
	}); err != nil {
		return err
	}

	// Deleted only once the rows are gone. Removing these inside the transaction
	// would leave deployments pointing at missing workflows if the commit failed.
	for _, raw := range orphanedWorkflows {
		if raw == "" {
			continue
		}
		if u, _ := url.Parse(raw); u != nil {
			if err := i.file.RemoveWorkflow(ctx, u); err != nil {
				log.Errorfc(ctx, "deployment: could not remove workflow %s of deleted deployment: %v", raw, err)
			}
		}
	}

	return nil
}

func (i *Deployment) Execute(ctx context.Context, p interfaces.ExecuteDeploymentParam) (_ *job.Job, err error) {
	ctx, span := otel.Tracer(tracerName).Start(ctx, "interactor.Deployment.Execute")
	span.SetAttributes(attribute.String("deployment.id", p.DeploymentID.String()))
	defer func() {
		if err != nil {
			span.RecordError(err)
		}
		span.End()
	}()

	dep, err := i.deploymentRepo.FindByID(ctx, p.DeploymentID)
	if err != nil {
		return nil, err
	}
	if dep == nil {
		return nil, fmt.Errorf("deployment not found: %s", p.DeploymentID)
	}
	if err := i.checkPermission(ctx, rbac.ActionAny, dep.Workspace()); err != nil {
		return nil, err
	}

	debug := false
	did := dep.ID()

	// Built before the transaction on purpose: job.New().NewID() inside a retried
	// closure mints a fresh ID on every attempt, so a serialization retry would
	// submit a second cloud job and orphan the first.
	j, err := job.New().
		NewID().
		Debug(&debug).
		Deployment(&did).
		Workspace(dep.Workspace()).
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

	var (
		workflowURL string
		projectID   id.ProjectID
	)
	if err := i.transaction.WithinTransaction(ctx, func(ctx context.Context) error {
		d, err := i.deploymentRepo.FindByID(ctx, p.DeploymentID)
		if err != nil {
			return err
		}
		if d == nil {
			return fmt.Errorf("deployment not found: %s", p.DeploymentID)
		}

		workflowURL = d.WorkflowURL()
		if d.Project() != nil {
			projectID = *d.Project()
		}

		return i.jobRepo.Save(ctx, j)
	}); err != nil {
		return nil, err
	}

	// Submitted only once the pending row is committed, so a failure here leaves a
	// job the user can see instead of a cloud job with no record of it.
	gcpJobID, err := i.batch.SubmitJob(ctx, j.ID(), workflowURL, j.MetadataURL(), nil, projectID, dep.Workspace(), nil, nil)
	if err != nil {
		failJob(ctx, i.jobRepo, j)
		return nil, interfaces.ErrJobCreationFailed
	}
	j.SetGCPJobID(gcpJobID)
	if err := i.jobRepo.Save(ctx, j); err != nil {
		return nil, err
	}

	if err := i.job.StartMonitoring(ctx, j, nil); err != nil {
		return nil, fmt.Errorf("failed to start job monitoring: %v", err)
	}

	return j, nil
}
