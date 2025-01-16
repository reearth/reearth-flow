package repo

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearthx/account/accountdomain"
)

type Trigger interface {
	Filtered(WorkspaceFilter) Trigger
	FindByIDs(context.Context, id.TriggerIDList) ([]*trigger.Trigger, error)
	FindByID(context.Context, id.TriggerID) (*trigger.Trigger, error)
	FindByWorkspace(context.Context, accountdomain.WorkspaceID) ([]*trigger.Trigger, error)
	Save(context.Context, *trigger.Trigger) error
	Remove(context.Context, id.TriggerID) error
}
