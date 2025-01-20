package interactor

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
	"github.com/reearth/reearthx/usecasex"
)

type Trigger struct {
	common
	triggerRepo    repo.Trigger
	deploymentRepo repo.Deployment
	workspaceRepo  accountrepo.Workspace
	transaction    usecasex.Transaction
}

func NewTrigger(r *repo.Container) interfaces.Trigger {
	return &Trigger{
		triggerRepo:    r.Trigger,
		deploymentRepo: r.Deployment,
		workspaceRepo:  r.Workspace,
		transaction:    r.Transaction,
	}
}

func (i *Trigger) Fetch(ctx context.Context, ids []id.TriggerID, operator *usecase.Operator) ([]*trigger.Trigger, error) {
	return i.triggerRepo.FindByIDs(ctx, ids)
}

func (i *Trigger) FindByWorkspace(ctx context.Context, id accountdomain.WorkspaceID, operator *usecase.Operator) ([]*trigger.Trigger, error) {
	return i.triggerRepo.FindByWorkspace(ctx, id)
}

func (i *Trigger) FindByID(ctx context.Context, id id.TriggerID, operator *usecase.Operator) (*trigger.Trigger, error) {
	return i.triggerRepo.FindByID(ctx, id)
}

func (i *Trigger) Create(ctx context.Context, param interfaces.CreateTriggerParam, operator *usecase.Operator) (result *trigger.Trigger, err error) {
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

	if _, err = i.deploymentRepo.FindByID(ctx, param.DeploymentID); err != nil {
		return nil, err
	}

	t := trigger.New().
		NewID().
		Workspace(param.WorkspaceID).
		Deployment(param.DeploymentID).
		EventSource(param.EventSource).
		UpdatedAt(time.Now())

	if param.EventSource == "TIME_DRIVEN" {
		t = t.TimeInterval(trigger.TimeInterval(param.TimeInterval))
	} else if param.EventSource == "API_DRIVEN" {
		t = t.AuthToken(param.AuthToken)
	}

	trg, err := t.Build()
	if err != nil {
		return nil, err
	}

	if err := i.triggerRepo.Save(ctx, trg); err != nil {
		return nil, err
	}

	tx.Commit()
	return trg, nil
}

func (i *Trigger) Update(ctx context.Context, param interfaces.UpdateTriggerParam, operator *usecase.Operator) (_ *trigger.Trigger, err error) {
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

	t, err := i.triggerRepo.FindByID(ctx, param.ID)
	if err != nil {
		return nil, err
	}

	if param.EventSource == "TIME_DRIVEN" {
		t.SetEventSource(trigger.EventSourceType(param.EventSource))
		t.SetTimeInterval(trigger.TimeInterval(param.TimeInterval))
		t.SetAuthToken("")
	} else if param.EventSource == "API_DRIVEN" {
		t.SetEventSource(trigger.EventSourceType(param.EventSource))
		t.SetTimeInterval("")
		t.SetAuthToken(param.AuthToken)
	}

	if err := i.triggerRepo.Save(ctx, t); err != nil {
		return nil, err
	}

	tx.Commit()
	return t, nil
}

func (i *Trigger) Delete(ctx context.Context, id id.TriggerID, operator *usecase.Operator) (err error) {
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

	if err := i.triggerRepo.Remove(ctx, id); err != nil {
		return err
	}

	tx.Commit()
	return nil
}
