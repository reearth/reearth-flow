package gateway

import (
	"context"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
)

// PolicyCheckType names a plan constraint. The values are shared with the other
// Re:Earth products and answered from the workspace's plan by the policy service.
type PolicyCheckType string

const (
	PolicyCheckPrivateProjectCreation PolicyCheckType = "general_private_project_creation"
	// PolicyCheckHourlySchedule asks whether a trigger may run every hour.
	PolicyCheckHourlySchedule PolicyCheckType = "flow_hourly_schedule"
	// PolicyCheckConcurrentJobCount asks whether the workspace may have Value
	// jobs pending or running at once, counting the new one.
	PolicyCheckConcurrentJobCount PolicyCheckType = "flow_concurrent_job_count"
	// PolicyCheckAPITriggerCallsPerDay asks whether the workspace's API
	// triggers may receive Value calls today, counting the new one.
	PolicyCheckAPITriggerCallsPerDay PolicyCheckType = "flow_api_trigger_calls_per_day"
)

// PolicyLimitUnlimited is the CurrentLimit of a check whose plan has no limit.
const PolicyLimitUnlimited = "unlimited"

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

// DefaultJobLifetime covers Batch's default six-hour run limit plus time to
// queue and start the worker.
const DefaultJobLifetime = 7 * time.Hour

// JobLifetimeFor is how long a Batch job can live: every attempt may run for
// maxRun, and queueing and VM start-up add up to jobStartupMargin on top.
func JobLifetimeFor(maxRun time.Duration, attempts int) time.Duration {
	if attempts < 1 {
		attempts = 1
	}
	return maxRun*time.Duration(attempts) + jobStartupMargin
}

const jobStartupMargin = time.Hour
