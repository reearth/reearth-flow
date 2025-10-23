package repo

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
)

type Trigger interface {
	Filtered(WorkspaceFilter) Trigger
	FindByID(context.Context, id.TriggerID) (*trigger.Trigger, error)
	FindByIDs(context.Context, id.TriggerIDList) ([]*trigger.Trigger, error)
	FindByWorkspace(context.Context, id.WorkspaceID, *interfaces.PaginationParam, *string) ([]*trigger.Trigger, *interfaces.PageBasedInfo, error)
	Remove(context.Context, id.TriggerID) error
	Save(context.Context, *trigger.Trigger) error
}
