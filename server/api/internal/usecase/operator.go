package usecase

import (
	"github.com/reearth/reearthx/account/accountusecase"
)

// TODO: After migrating to Cerbos for permission management and modifying reearthx and reearth-dashboard interfaces,
// this file and all its usages will be deleted.
type Operator struct {
	AcOperator *accountusecase.Operator
}
