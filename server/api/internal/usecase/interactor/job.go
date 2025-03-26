package interactor

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/job/monitor"
	"github.com/reearth/reearth-flow/api/pkg/notification"
	"github.com/reearth/reearth-flow/api/pkg/subscription"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/usecasex"
)

type Job struct {
	jobRepo           repo.Job
	workspaceRepo     accountrepo.Workspace
	transaction       usecasex.Transaction
	file              gateway.File
	batch             gateway.Batch
	monitor           *monitor.Monitor
	subscriptions     *subscription.JobManager
	notifier          notification.Notifier
	permissionChecker gateway.PermissionChecker
	monitoringMu      sync.RWMutex
	monitoringJobs    map[string]time.Time
}

type NotificationPayload struct {
	RunID        string   `json:"runId"`
	DeploymentID string   `json:"deploymentId"`
	Status       string   `json:"status"`
	Logs         []string `json:"logs"`
	Outputs      []string `json:"outputs"`
}

func NewJob(r *repo.Container, gr *gateway.Container, permissionChecker gateway.PermissionChecker) interfaces.Job {
	return &Job{
		jobRepo:           r.Job,
		workspaceRepo:     r.Workspace,
		transaction:       r.Transaction,
		file:              gr.File,
		batch:             gr.Batch,
		monitor:           monitor.NewMonitor(),
		subscriptions:     subscription.NewJobManager(),
		notifier:          notification.NewHTTPNotifier(),
		permissionChecker: permissionChecker,
		monitoringMu:      sync.RWMutex{},
		monitoringJobs:    make(map[string]time.Time),
	}
}

func (i *Job) checkPermission(ctx context.Context, action string) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceJob, action)
}

func (i *Job) Cancel(ctx context.Context, jobID id.JobID) (*job.Job, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	j, err := i.jobRepo.FindByID(ctx, jobID)
	if err != nil {
		return nil, err
	}

	if j.Status() != job.StatusPending && j.Status() != job.StatusRunning {
		return nil, fmt.Errorf("job cannot be cancelled: current status is %s", j.Status())
	}

	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return nil, err
	}
	defer func() {
		if err := tx.End(ctx); err != nil {
			log.Errorfc(ctx, "transaction end failed: %v", err)
		}
	}()

	if err := i.batch.CancelJob(ctx, j.GCPJobID()); err != nil {
		return nil, err
	}

	j.SetStatus(job.StatusCancelled)
	now := time.Now()
	j.SetCompletedAt(&now)

	if err := i.jobRepo.Save(ctx, j); err != nil {
		return nil, err
	}

	tx.Commit()

	if err := i.handleJobCompletion(ctx, j); err != nil {
		log.Errorfc(ctx, "job: completion handling failed: %v", err)
	}

	i.subscriptions.Notify(j.ID().String(), j.Status())
	i.monitor.Remove(j.ID().String())

	return j, nil
}

func (i *Job) FindByID(ctx context.Context, id id.JobID) (*job.Job, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	j, err := i.jobRepo.FindByID(ctx, id)
	if err != nil {
		return nil, err
	}
	return j, nil
}

func (i *Job) Fetch(ctx context.Context, ids []id.JobID) ([]*job.Job, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	jobs, err := i.jobRepo.FindByIDs(ctx, ids)
	if err != nil {
		return nil, err
	}
	return jobs, nil
}

func (i *Job) FindByWorkspace(ctx context.Context, wsID accountdomain.WorkspaceID, p *interfaces.PaginationParam) ([]*job.Job, *interfaces.PageBasedInfo, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, nil, err
	}

	return i.jobRepo.FindByWorkspace(ctx, wsID, p)
}

func (i *Job) GetStatus(ctx context.Context, jobID id.JobID) (job.Status, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return "", err
	}

	j, err := i.jobRepo.FindByID(ctx, jobID)
	if err != nil {
		return "", err
	}
	return j.Status(), nil
}
