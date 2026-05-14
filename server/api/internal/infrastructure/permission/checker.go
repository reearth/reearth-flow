package permission

import (
	"context"

	"github.com/reearth/reearth-accounts/server/pkg/gqlclient/cerbos"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
)

type checker struct {
	repo    cerbos.Repo
	service string
}

func NewChecker(repo cerbos.Repo, service string) gateway.PermissionChecker {
	return &checker{repo: repo, service: service}
}

func (c *checker) CheckPermission(ctx context.Context, resource string, action string) (bool, error) {
	result, err := c.repo.CheckPermission(ctx, cerbos.CheckPermissionParam{
		Service:  c.service,
		Resource: resource,
		Action:   action,
	})
	if err != nil {
		return false, err
	}
	return result.Allowed, nil
}
