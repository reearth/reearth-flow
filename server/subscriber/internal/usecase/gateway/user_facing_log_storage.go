package gateway

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/pkg/userfacinglog"
)

type UserFacingLogStorage interface {
	Save(ctx context.Context, event *userfacinglog.UserFacingLogEvent) error
}
