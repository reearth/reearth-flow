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
	log.Debugfc(ctx, "job: starting monitoring for jobID=%s workspace=%s", j.ID(), j.Workspace())

	i.monitoringMu.Lock()
	if cancel, exists := i.monitoring[j.ID().String()]; exists {
		log.Debugfc(ctx, "job: cancelling existing monitoring for jobID=%s", j.ID())
		cancel()
	}
	monitorCtx, cancel := context.WithCancel(context.Background())
	i.monitoring[j.ID().String()] = cancel
	i.monitoringMu.Unlock()

	log.Debugfc(ctx, "job: initialized monitoring context for jobID=%s", j.ID())

	go func() {
		ticker := time.NewTicker(10 * time.Second)
		defer ticker.Stop()
		defer cancel()

		log.Debugfc(ctx, "job: started monitoring loop for jobID=%s", j.ID())

		for {
			select {
			case <-monitorCtx.Done():
				log.Debugfc(ctx, "job: monitoring context cancelled for jobID=%s", j.ID())
				return
			case <-ticker.C:
				checkCtx := context.Background()
				log.Debugfc(checkCtx, "job: checking status for jobID=%s gcpJobID=%s", j.ID(), j.GCPJobID())

				status, err := i.batch.GetJobStatus(checkCtx, j.GCPJobID())
				if err != nil {
					log.Errorfc(checkCtx, "job: failed to get job status: jobID=%s error=%v", j.ID(), err)
					continue
				}
				log.Debugfc(checkCtx, "job: received status=%s for jobID=%s", status, j.ID())

				if j.Status() != job.Status(status) {
					log.Debugfc(checkCtx, "job: status changed from %s to %s for jobID=%s", j.Status(), status, j.ID())

					tx, err := i.transaction.Begin(checkCtx)
					if err != nil {
						log.Errorfc(checkCtx, "job: failed to begin transaction: jobID=%s error=%v", j.ID(), err)
						continue
					}

					txCtx := tx.Context()

					j.SetStatus(job.Status(status))
					if err := i.jobRepo.Save(txCtx, j); err != nil {
						log.Errorfc(txCtx, "job: failed to save job: jobID=%s error=%v", j.ID(), err)
						if endErr := tx.End(txCtx); endErr != nil {
							log.Errorfc(txCtx, "job: failed to end transaction after save error: jobID=%s error=%v", j.ID(), endErr)
						}
						continue
					}

					tx.Commit()
					log.Debugfc(txCtx, "job: committed status update transaction for jobID=%s", j.ID())

					if err := tx.End(txCtx); err != nil {
						log.Errorfc(txCtx, "job: failed to end transaction: jobID=%s error=%v", j.ID(), err)
						continue
					}

					log.Debugfc(txCtx, "job: notifying subscribers of status change for jobID=%s status=%s", j.ID(), status)
					i.notifySubscribers(j.ID().String(), j.Status())
				}

				if status == gateway.JobStatusCompleted || status == gateway.JobStatusFailed {
					log.Debugfc(checkCtx, "job: stopping monitoring due to terminal status=%s for jobID=%s", status, j.ID())
					i.stopMonitoring(j.ID().String())
					return
				}
			}
		}
	}()

	return nil
}

func (i *Job) Subscribe(ctx context.Context, jobID id.JobID, operator *usecase.Operator) (chan job.Status, error) {
	log.Debugfc(ctx, "job: new subscription request for jobID=%s", jobID)

	j, err := i.FindByID(ctx, jobID, operator)
	if err != nil {
		log.Debugfc(ctx, "job: failed to find job for subscription: jobID=%s error=%v", jobID, err)
		return nil, err
	}

	ch := make(chan job.Status, 1)

	i.subscribersMu.Lock()
	i.subscribers[jobID.String()] = append(i.subscribers[jobID.String()], ch)
	log.Debugfc(ctx, "job: added new subscriber for jobID=%s total_subscribers=%d", jobID, len(i.subscribers[jobID.String()]))
	i.subscribersMu.Unlock()

	log.Debugfc(ctx, "job: sending initial status=%s to new subscriber for jobID=%s", j.Status(), jobID)
	ch <- j.Status()

	return ch, nil
}

func (i *Job) Unsubscribe(jobID id.JobID, ch chan job.Status) {
	log.Debugfc(context.Background(), "job: unsubscribe request for jobID=%s", jobID)

	i.subscribersMu.Lock()
	defer i.subscribersMu.Unlock()

	subs := i.subscribers[jobID.String()]
	for idx, sub := range subs {
		if sub == ch {
			close(sub)
			i.subscribers[jobID.String()] = append(subs[:idx], subs[idx+1:]...)
			log.Debugfc(context.Background(), "job: removed subscriber for jobID=%s remaining_subscribers=%d", jobID, len(i.subscribers[jobID.String()]))
			break
		}
	}
}

func (i *Job) notifySubscribers(jobID string, status job.Status) {
	i.subscribersMu.RLock()
	defer i.subscribersMu.RUnlock()

	subscriberCount := len(i.subscribers[jobID])
	log.Debugfc(context.Background(), "job: notifying %d subscribers for jobID=%s status=%s", subscriberCount, jobID, status)

	for _, ch := range i.subscribers[jobID] {
		select {
		case ch <- status:
			log.Debugfc(context.Background(), "job: successfully sent status update to subscriber for jobID=%s", jobID)
		default:
			log.Debugfc(context.Background(), "job: skipped blocked subscriber for jobID=%s", jobID)
		}
	}
}

func (i *Job) stopMonitoring(jobID string) {
	log.Debugfc(context.Background(), "job: stopping monitoring for jobID=%s", jobID)

	i.monitoringMu.Lock()
	if cancel, exists := i.monitoring[jobID]; exists {
		cancel()
		delete(i.monitoring, jobID)
		log.Debugfc(context.Background(), "job: removed monitoring context for jobID=%s", jobID)
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
