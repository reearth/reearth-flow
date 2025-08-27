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

// TODO: After migration, remove accountinterfaces.User and rename tempNewUsecase to usecase.
type UserLoader struct {
	usecase        accountinterfaces.User
	tempNewUsecase interfaces.User
}

func NewUserLoader(usecase accountinterfaces.User, tempNewUsecase interfaces.User) *UserLoader {
	return &UserLoader{
		usecase:        usecase,
		tempNewUsecase: tempNewUsecase,
	}
}

// TODO: After migration, remove this logic and use the new usecase directly.
func (c *UserLoader) Fetch(ctx context.Context, ids []gqlmodel.ID) ([]*gqlmodel.User, []error) {
	if c.tempNewUsecase != nil {
		users := c.fetchWithTempNewUsecase(ctx, ids)
		if len(users) > 0 {
			log.Printf("DEBUG:[UserLoader.Fetch] Fetched %d users with tempNewUsecase", len(users))
			return users, nil
		}
	}
	log.Printf("WARNING:[UserLoader.Fetch] Fallback to traditional usecase for %d IDs", len(ids))
	return c.fetchWithTraditionalUsecase(ctx, ids)
}

func (c *UserLoader) fetchWithTempNewUsecase(ctx context.Context, ids []gqlmodel.ID) []*gqlmodel.User {
	uids, err := util.TryMap(ids, gqlmodel.ToID[id.User])
	if err != nil {
		log.Printf("WARNING:[UserLoader.fetchWithTempNewUsecase] Failed to convert IDs: %v", err)
		return nil
	}

	res, err := c.tempNewUsecase.FindByIDs(ctx, uids)
	if err != nil {
		log.Printf("WARNING:[UserLoader.fetchWithTempNewUsecase] Failed to find users: %v", err)
		return nil
	}

	if len(res) == 0 {
		log.Printf("DEBUG:[UserLoader.fetchWithTempNewUsecase] No users found for IDs: %v", ids)
		return nil
	}

	users := make([]*gqlmodel.User, 0, len(res))
	for _, u := range res {
		users = append(users, gqlmodel.ToUserFromFlow(u))
	}

	return users
}

func (c *UserLoader) fetchWithTraditionalUsecase(ctx context.Context, ids []gqlmodel.ID) ([]*gqlmodel.User, []error) {
	uids, err := util.TryMap(ids, gqlmodel.ToID[accountdomain.User])
	if err != nil {
		return nil, []error{err}
	}

	res, err := c.usecase.FetchByID(ctx, uids)
	if err != nil {
		return nil, []error{err}
	}

	users := make([]*gqlmodel.User, 0, len(res))
	for _, u := range res {
		users = append(users, gqlmodel.ToUser(u))
	}

	return users, nil
}

// TODO: After migration, remove this logic and use the new usecase directly.
func (c *UserLoader) SearchUser(ctx context.Context, nameOrEmail string) (*gqlmodel.User, error) {
	if c.tempNewUsecase != nil {
		u := c.searchUserWithTempNewUsecase(ctx, nameOrEmail)
		if u != nil {
			log.Printf("DEBUG:[UserLoader.SearchUser] Fetched user with tempNewUsecase id: %s", u.ID)
			return u, nil
		}
	}
	log.Printf("WARNING:[UserLoader.SearchUser] Fallback to traditional usecase for search")

	res, err := c.usecase.SearchUser(ctx, nameOrEmail)
	if err != nil {
		return nil, err
	}

	users := gqlmodel.ToUsersFromSimple(res)
	if len(users) == 0 {
		return nil, nil
	}
	return users[0], nil
}

func (c *UserLoader) searchUserWithTempNewUsecase(ctx context.Context, nameOrEmail string) *gqlmodel.User {
	res, err := c.tempNewUsecase.UserByNameOrEmail(ctx, nameOrEmail)
	if err != nil {
		log.Printf("WARNING:[UserLoader.SearchUser] Failed to search users: %v", err)
		return nil
	}
	if res == nil {
		log.Printf("DEBUG:[UserLoader.SearchUser] No user found for nameOrEmail: %s", nameOrEmail)
		return nil
	}

	return gqlmodel.ToUserFromFlow(res)
}

// data loader

type UserDataLoader interface {
	Load(gqlmodel.ID) (*gqlmodel.User, error)
	LoadAll([]gqlmodel.ID) ([]*gqlmodel.User, []error)
}

func (c *UserLoader) DataLoader(ctx context.Context) UserDataLoader {
	return gqldataloader.NewUserLoader(gqldataloader.UserLoaderConfig{
		Wait:     dataLoaderWait,
		MaxBatch: dataLoaderMaxBatch,
		Fetch: func(keys []gqlmodel.ID) ([]*gqlmodel.User, []error) {
			return c.Fetch(ctx, keys)
		},
	})
}

func (c *UserLoader) OrdinaryDataLoader(ctx context.Context) UserDataLoader {
	return &ordinaryUserLoader{
		fetch: func(keys []gqlmodel.ID) ([]*gqlmodel.User, []error) {
			return c.Fetch(ctx, keys)
		},
	}
}

type ordinaryUserLoader struct {
	fetch func(keys []gqlmodel.ID) ([]*gqlmodel.User, []error)
}

func (l *ordinaryUserLoader) Load(key gqlmodel.ID) (*gqlmodel.User, error) {
	res, errs := l.fetch([]gqlmodel.ID{key})
	if len(errs) > 0 {
		return nil, errs[0]
	}
	if len(res) > 0 {
		return res[0], nil
	}
	return nil, nil
}

func (l *ordinaryUserLoader) LoadAll(keys []gqlmodel.ID) ([]*gqlmodel.User, []error) {
	return l.fetch(keys)
}
