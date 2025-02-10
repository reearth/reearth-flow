package gql

// import (
// 	"context"

// 	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
// 	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
// )

// type ProjectAccessLoader struct {
// 	usecase interfaces.ProjectAccess
// }

// func NewProjectAccessLoader(usecase interfaces.ProjectAccess) *ProjectAccessLoader {
// 	return &ProjectAccessLoader{usecase: usecase}
// }

// func (c *ProjectAccessLoader) Fetch(ctx context.Context, id gqlmodel.ID) (*gqlmodel.Project, error) {
// 	res, err := c.usecase.Fetch(ctx, id, getOperator(ctx))
// 	if err != nil {
// 		return nil, err
// 	}

// 	return res, nil
// }
