package permission

import (
	"context"
	"fmt"

	"github.com/reearth/reearth-flow/api/internal/adapter"
	cerbosClient "github.com/reearth/reearthx/cerbos/client"
)

type PermissionChecker struct {
	Service      string
	DashboardURL string
}

func NewPermissionChecker(service string, dashboardURL string) *PermissionChecker {
	return &PermissionChecker{
		Service:      service,
		DashboardURL: dashboardURL,
	}
}

func (p *PermissionChecker) CheckPermission(ctx context.Context, resource string, action string) (bool, error) {
	authInfo := adapter.GetAuthInfo(ctx)
	if authInfo == nil {
		return false, fmt.Errorf("auth info not found")
	}

	input := cerbosClient.CheckPermissionInput{
		Service:  p.Service,
		Resource: resource,
		Action:   action,
	}

	client := cerbosClient.NewClient(p.DashboardURL)
	return client.CheckPermission(ctx, authInfo, input)
}
