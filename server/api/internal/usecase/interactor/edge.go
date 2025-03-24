package interactor

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/graph"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/usecasex"
)

type EdgeExecution struct {
	file              gateway.File
	edgeRepo          repo.EdgeExecution
	transaction       usecasex.Transaction
	permissionChecker gateway.PermissionChecker
}

func NewEdgeExecution(r *repo.Container, gr *gateway.Container, permissionChecker gateway.PermissionChecker) interfaces.EdgeExecution {
	ee := &EdgeExecution{
		edgeRepo:          r.EdgeExecution,
		file:              gr.File,
		transaction:       r.Transaction,
		permissionChecker: permissionChecker,
	}
	return ee
}

func (i *EdgeExecution) checkPermission(ctx context.Context, action string) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceJob, action)
}

func (i *EdgeExecution) FindByJobEdgeID(ctx context.Context, jobID id.JobID, edgeID string) (*graph.EdgeExecution, error) {
	if err := i.checkPermission(ctx, rbac.ActionAny); err != nil {
		return nil, err
	}

	edge, err := i.edgeRepo.FindByJobEdgeID(ctx, jobID, edgeID)
	if err != nil {
		return nil, err
	}

	if edge != nil && edge.IntermediateDataURL() == nil {
		if err := i.checkIntermediateData(ctx, edge); err != nil {
			log.Errorfc(ctx, "edgeExecution: failed to check intermediate data: %v", err)
		}
	}

	return edge, nil
}

func (i *EdgeExecution) checkIntermediateData(ctx context.Context, edge *graph.EdgeExecution) error {
	edgeID := edge.ID().String()
	jobID := edge.JobID().String()

	exists, err := i.file.CheckIntermediateDataExists(ctx, edgeID, jobID)
	if err != nil {
		return err
	}

	if !exists {
		return nil
	}

	url := i.file.GetIntermediateDataURL(ctx, edgeID, jobID)
	if url == "" {
		return nil
	}

	tx, err := i.transaction.Begin(ctx)
	if err != nil {
		return err
	}

	defer func() {
		if err := tx.End(ctx); err != nil {
			log.Errorfc(ctx, "transaction end failed: %v", err)
		}
	}()

	newEdge := graph.NewEdgeExecution(
		edge.ID(),
		edge.EdgeID(),
		edge.JobID(),
		&url,
	)

	if err := i.edgeRepo.Save(ctx, newEdge); err != nil {
		return err
	}

	tx.Commit()

	*edge = *newEdge

	return nil
}
