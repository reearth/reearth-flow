package gql

import (
	"context"
	"log"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqldataloader"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/account/accountusecase/accountinterfaces"
	"github.com/reearth/reearthx/util"
)

// TODO: After migration, remove accountinterfaces.Workspace and rename tempNewUsecase to usecase.
type WorkspaceLoader struct {
	usecase        accountinterfaces.Workspace
	tempNewUsecase interfaces.Workspace
}

func NewWorkspaceLoader(usecase accountinterfaces.Workspace, tempNewUsecase interfaces.Workspace) *WorkspaceLoader {
	return &WorkspaceLoader{
		usecase:        usecase,
		tempNewUsecase: tempNewUsecase,
	}
}

// TODO: After migration, remove this logic and use the new usecase directly.
func (c *WorkspaceLoader) Fetch(ctx context.Context, ids []gqlmodel.ID) ([]*gqlmodel.Workspace, []error) {
	if c.tempNewUsecase != nil {
		workspaces := c.fetchWithTempNewUsecase(ctx, ids)
		if len(workspaces) > 0 {
			log.Printf("DEBUG:[WorkspaceLoader.Fetch] Fetched %d workspaces with tempNewUsecase", len(workspaces))
			return workspaces, nil
		}
	}
	log.Printf("WARNING:[WorkspaceLoader.Fetch] Fallback to traditional usecase for %d IDs", len(ids))
	return c.fetchWithTraditionalUsecase(ctx, ids)
}

func (c *WorkspaceLoader) fetchWithTempNewUsecase(ctx context.Context, ids []gqlmodel.ID) []*gqlmodel.Workspace {
	wids, err := util.TryMap(ids, gqlmodel.ToID[id.Workspace])
	if err != nil {
		log.Printf("WARNING:[WorkspaceLoader.fetchWithTempNewUsecase] Failed to convert IDs: %v", err)
		return nil
	}

	res, err := c.tempNewUsecase.FindByIDs(ctx, wids)
	if err != nil {
		log.Printf("WARNING:[WorkspaceLoader.fetchWithTempNewUsecase] Failed to find workspaces: %v", err)
		return nil
	}

	if len(res) == 0 {
		log.Printf("DEBUG:[WorkspaceLoader.fetchWithTempNewUsecase] No workspaces found for IDs: %v", ids)
		return nil
	}

	workspaces := make([]*gqlmodel.Workspace, 0, len(res))
	for _, t := range res {
		workspaces = append(workspaces, gqlmodel.ToWorkspaceFromFlow(t))
	}
	return workspaces
}

func (c *WorkspaceLoader) fetchWithTraditionalUsecase(ctx context.Context, ids []gqlmodel.ID) ([]*gqlmodel.Workspace, []error) {
	uids, err := util.TryMap(ids, gqlmodel.ToID[accountdomain.Workspace])
	if err != nil {
		return nil, []error{err}
	}

	res, err := c.usecase.Fetch(ctx, uids, getAcOperator(ctx))
	if err != nil {
		return nil, []error{err}
	}

	workspaces := make([]*gqlmodel.Workspace, 0, len(res))
	for _, t := range res {
		workspaces = append(workspaces, gqlmodel.ToWorkspace(t))
	}
	return workspaces, nil
}

// TODO: After migration, remove this logic and use the new usecase directly.
func (c *WorkspaceLoader) FindByUser(ctx context.Context, uid gqlmodel.ID) ([]*gqlmodel.Workspace, error) {
	if c.tempNewUsecase != nil {
		workspaces := c.findByUserWithTempNewUsecase(ctx, uid)
		if len(workspaces) > 0 {
			log.Printf("DEBUG:[WorkspaceLoader.FindByUser] Fetched %d workspaces with tempNewUsecase", len(workspaces))
			return workspaces, nil
		}
	}
	log.Printf("WARNING:[WorkspaceLoader.FindByUser] Fallback to traditional usecase for %s", uid)
	return c.findByUserWithTraditionalUsecase(ctx, uid)
}

func (c *WorkspaceLoader) findByUserWithTempNewUsecase(ctx context.Context, uid gqlmodel.ID) []*gqlmodel.Workspace {
	tid, err := gqlmodel.ToID[id.User](uid)
	if err != nil {
		log.Printf("WARNING:[WorkspaceLoader.findByUserWithTempNewUsecase] Failed to convert ID: %v", err)
		return nil
	}

	res, err := c.tempNewUsecase.FindByUser(ctx, tid)
	if err != nil {
		log.Printf("WARNING:[WorkspaceLoader.findByUserWithTempNewUsecase] Failed to find workspaces: %v", err)
		return nil
	}

	if len(res) == 0 {
		log.Printf("DEBUG:[WorkspaceLoader.findByUserWithTempNewUsecase] No workspaces found for ID: %s", tid)
		return nil
	}

	workspaces := make([]*gqlmodel.Workspace, 0, len(res))
	for _, t := range res {
		workspaces = append(workspaces, gqlmodel.ToWorkspaceFromFlow(t))
	}
	return workspaces
}

func (c *WorkspaceLoader) findByUserWithTraditionalUsecase(ctx context.Context, uid gqlmodel.ID) ([]*gqlmodel.Workspace, error) {
	userid, err := gqlmodel.ToID[accountdomain.User](uid)
	if err != nil {
		return nil, err
	}

	res, err := c.usecase.FindByUser(ctx, userid, getAcOperator(ctx))
	if err != nil {
		return nil, err
	}
	workspaces := make([]*gqlmodel.Workspace, 0, len(res))
	for _, t := range res {
		workspaces = append(workspaces, gqlmodel.ToWorkspace(t))
	}
	return workspaces, nil
}

// data loader

type WorkspaceDataLoader interface {
	Load(gqlmodel.ID) (*gqlmodel.Workspace, error)
	LoadAll([]gqlmodel.ID) ([]*gqlmodel.Workspace, []error)
}

func (c *WorkspaceLoader) DataLoader(ctx context.Context) WorkspaceDataLoader {
	return gqldataloader.NewWorkspaceLoader(gqldataloader.WorkspaceLoaderConfig{
		Wait:     dataLoaderWait,
		MaxBatch: dataLoaderMaxBatch,
		Fetch: func(keys []gqlmodel.ID) ([]*gqlmodel.Workspace, []error) {
			return c.Fetch(ctx, keys)
		},
	})
}

func (c *WorkspaceLoader) OrdinaryDataLoader(ctx context.Context) WorkspaceDataLoader {
	return &ordinaryWorkspaceLoader{
		fetch: func(keys []gqlmodel.ID) ([]*gqlmodel.Workspace, []error) {
			return c.Fetch(ctx, keys)
		},
	}
}

type ordinaryWorkspaceLoader struct {
	fetch func(keys []gqlmodel.ID) ([]*gqlmodel.Workspace, []error)
}

func (l *ordinaryWorkspaceLoader) Load(key gqlmodel.ID) (*gqlmodel.Workspace, error) {
	res, errs := l.fetch([]gqlmodel.ID{key})
	if len(errs) > 0 {
		return nil, errs[0]
	}
	if len(res) > 0 {
		return res[0], nil
	}
	return nil, nil
}

func (l *ordinaryWorkspaceLoader) LoadAll(keys []gqlmodel.ID) ([]*gqlmodel.Workspace, []error) {
	return l.fetch(keys)
}
