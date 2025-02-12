package asset

import (
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
)

type (
	ID          = id.AssetID
	WorkspaceID = accountdomain.WorkspaceID
)

var (
	NewID          = id.NewAssetID
	NewWorkspaceID = accountdomain.NewWorkspaceID
)

var (
	MustID          = id.MustAssetID
	MustWorkspaceID = id.MustWorkspaceID
)

var (
	IDFrom          = id.AssetIDFrom
	WorkspaceIDFrom = accountdomain.WorkspaceIDFrom
)

var (
	IDFromRef          = id.AssetIDFromRef
	WorkspaceIDFromRef = accountdomain.WorkspaceIDFromRef
)

var ErrInvalidID = id.ErrInvalidID

func MockNewID(i ID) func() {
	NewID = func() ID { return i }
	return func() { NewID = id.NewAssetID }
}
