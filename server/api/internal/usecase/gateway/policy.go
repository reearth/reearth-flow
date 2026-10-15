package gateway

import (
	"context"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
)

// PolicyCheckType names a plan constraint. The values are shared with the other
// Re:Earth products and answered from the workspace's plan by the policy service.
type PolicyCheckType string

const (
	PolicyCheckPrivateProjectCreation PolicyCheckType = "general_private_project_creation"
)

type PolicyCheckRequest struct {
	CheckType   PolicyCheckType        `json:"check_type"`
	WorkspaceID accountsid.WorkspaceID `json:"workspace_id"`
	Value       int64                  `json:"value"`
}

type PolicyCheckResponse struct {
	CheckType    PolicyCheckType `json:"check_type"`
	CurrentLimit string          `json:"current_limit"`
	Message      string          `json:"message"`
	Value        int64           `json:"value"`
	Allowed      bool            `json:"allowed"`
}

// PolicyChecker checks a workspace's plan constraints. Deployments without a
// policy service use a permissive checker that allows everything.
type PolicyChecker interface {
	CheckPolicy(context.Context, PolicyCheckRequest) (*PolicyCheckResponse, error)
}
