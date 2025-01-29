package repo

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/usecasex"
)

type Trigger interface {
	Filtered(WorkspaceFilter) Trigger
	FindByID(context.Context, id.TriggerID) (*trigger.Trigger, error)
	FindByIDs(context.Context, id.TriggerIDList) ([]*trigger.Trigger, error)
	FindByWorkspace(context.Context, accountdomain.WorkspaceID, *usecasex.Pagination) ([]*trigger.Trigger, *usecasex.PageInfo, error)
	Remove(context.Context, id.TriggerID) error
	FindByID(context.Context, id.TriggerID) (*trigger.Trigger, error)
	FindByWorkspace(context.Context, accountdomain.WorkspaceID, *interfaces.PaginationParam) ([]*trigger.Trigger, *usecasex.PageInfo, error)
	Save(context.Context, *trigger.Trigger) error
}
