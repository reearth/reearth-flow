package interactor

import (
	"context"
	"sync"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/usecasex"
)

type Job struct {
	common
	jobRepo       repo.Job
	workspaceRepo accountrepo.Workspace
	transaction   usecasex.Transaction
	batch         gateway.Batch
	subscribers   map[string][]chan job.Status
	subscribersMu sync.RWMutex
	monitoring    map[string]context.CancelFunc
	monitoringMu  sync.RWMutex
}

func NewJob(r *repo.Container, gr *gateway.Container) interfaces.Job {
	return &Job{
		jobRepo:       r.Job,
		workspaceRepo: r.Workspace,
		transaction:   r.Transaction,
		batch:         gr.Batch,
		subscribers:   make(map[string][]chan job.Status),
		monitoring:    make(map[string]context.CancelFunc),
	}
}

func (i *Job) FindByID(ctx context.Context, id id.JobID, operator *usecase.Operator) (*job.Job, error) {
	j, err := i.jobRepo.FindByID(ctx, id)
	if err != nil {
		return nil, err
	}
	if err := i.CanReadWorkspace(j.Workspace(), operator); err != nil {
		return nil, err
	}
	return j, nil
}

func (i *Job) Fetch(ctx context.Context, ids []id.JobID, operator *usecase.Operator) ([]*job.Job, error) {
	jobs, err := i.jobRepo.FindByIDs(ctx, ids)
	if err != nil {
		return nil, err
	}
	return i.filterReadableJobs(jobs, operator), nil
}

func (i *Job) FindByWorkspace(ctx context.Context, wsID accountdomain.WorkspaceID, pagination *usecasex.Pagination, operator *usecase.Operator) ([]*job.Job, *usecasex.PageInfo, error) {
	if err := i.CanReadWorkspace(wsID, operator); err != nil {
		return nil, nil, err
	}
	return i.jobRepo.FindByWorkspace(ctx, wsID, pagination)
}

func (i *Job) GetStatus(ctx context.Context, jobID id.JobID, operator *usecase.Operator) (job.Status, error) {
	j, err := i.jobRepo.FindByID(ctx, jobID)
	if err != nil {
		return "", err
	}
	if err := i.CanReadWorkspace(j.Workspace(), operator); err != nil {
		return "", err
	}
	return j.Status(), nil
}

func (i *Job) StartMonitoring(ctx context.Context, j *job.Job, operator *usecase.Operator) error {
	if err := i.CanReadWorkspace(j.Workspace(), operator); err != nil {
		return err
	}

	i.monitoringMu.Lock()
	if cancel, exists := i.monitoring[j.ID().String()]; exists {
		cancel()
	}
	monitorCtx, cancel := context.WithCancel(context.Background())
	i.monitoring[j.ID().String()] = cancel
	i.monitoringMu.Unlock()

	go func() {
		ticker := time.NewTicker(10 * time.Second)
		defer ticker.Stop()
		defer cancel()

		for {
			select {
			case <-monitorCtx.Done():
				return
			case <-ticker.C:
				checkCtx := context.Background()

				status, err := i.batch.GetJobStatus(checkCtx, j.GCPJobID())
				if err != nil {
					log.Errorfc(checkCtx, "failed to get job status: %v", err)
					continue
				}

				if j.Status() != job.Status(status) {
					tx, err := i.transaction.Begin(checkCtx)
					if err != nil {
						log.Errorfc(checkCtx, "failed to begin transaction: %v", err)
						continue
					}

					txCtx := tx.Context()

					j.SetStatus(job.Status(status))
					if err := i.jobRepo.Save(txCtx, j); err != nil {
						log.Errorfc(txCtx, "failed to save job: %v", err)
						if endErr := tx.End(txCtx); endErr != nil {
							log.Errorfc(txCtx, "failed to end transaction after save error: %v", endErr)
						}
						continue
					}

					tx.Commit()

					if err := tx.End(txCtx); err != nil {
						log.Errorfc(txCtx, "failed to end transaction: %v", err)
						continue
					}

					i.notifySubscribers(j.ID().String(), j.Status())
				}

				if status == gateway.JobStatusCompleted || status == gateway.JobStatusFailed {
					i.stopMonitoring(j.ID().String())
					return
				}
			}
		}
	}()

	return nil
}

func (i *Job) Subscribe(ctx context.Context, jobID id.JobID, operator *usecase.Operator) (chan job.Status, error) {
	j, err := i.FindByID(ctx, jobID, operator)
	if err != nil {
		return nil, err
	}

	ch := make(chan job.Status, 1)

	i.subscribersMu.Lock()
	i.subscribers[jobID.String()] = append(i.subscribers[jobID.String()], ch)
	i.subscribersMu.Unlock()

	// Send initial status
	ch <- j.Status()

	return ch, nil
}

func (i *Job) Unsubscribe(jobID id.JobID, ch chan job.Status) {
	i.subscribersMu.Lock()
	defer i.subscribersMu.Unlock()

	subs := i.subscribers[jobID.String()]
	for idx, sub := range subs {
		if sub == ch {
			close(sub)
			i.subscribers[jobID.String()] = append(subs[:idx], subs[idx+1:]...)
			break
		}
	}
}

func (i *Job) notifySubscribers(jobID string, status job.Status) {
	i.subscribersMu.RLock()
	defer i.subscribersMu.RUnlock()

	for _, ch := range i.subscribers[jobID] {
		select {
		case ch <- status:
		default:
		}
	}
}

func (i *Job) stopMonitoring(jobID string) {
	i.monitoringMu.Lock()
	if cancel, exists := i.monitoring[jobID]; exists {
		cancel()
		delete(i.monitoring, jobID)
	}
	i.monitoringMu.Unlock()
}

func (i *Job) filterReadableJobs(jobs []*job.Job, operator *usecase.Operator) []*job.Job {
	result := make([]*job.Job, 0, len(jobs))
	for _, j := range jobs {
		if i.CanReadWorkspace(j.Workspace(), operator) == nil {
			result = append(result, j)
		}
	}
	return result
}
