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

// FreeTierLimits bounds what a workspace on the free plan may run. Flow treats
// a workspace whose plan does not allow private projects as free, which is how
// the dashboard defines its free plan. The zero value imposes no limit.
type FreeTierLimits struct {
	// ConcurrentJobs caps the jobs a free workspace may have pending or running
	// at once. 0 means no cap.
	ConcurrentJobs int
	// DisallowHourly refuses hourly schedules on free workspaces.
	DisallowHourly bool
}

// Enabled reports whether any limit is set, so callers can skip asking the
// policy service when there is nothing to enforce.
func (l FreeTierLimits) Enabled() bool {
	return l.ConcurrentJobs > 0 || l.DisallowHourly
}
