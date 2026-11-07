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
	// PolicyCheckAPITriggerCallsPerDay asks whether the workspace's API
	// triggers may receive Value calls today, counting the new one.
	PolicyCheckAPITriggerCallsPerDay PolicyCheckType = "flow_api_trigger_calls_per_day"
	// PolicyCheckAssetSize asks whether the workspace may upload an asset of
	// Value bytes.
	PolicyCheckAssetSize PolicyCheckType = "flow_asset_size"
	// PolicyCheckConcurrentRenderCount asks whether the workspace may have
	// Value intermediate-data renders in progress at once, counting the new one.
	PolicyCheckConcurrentRenderCount PolicyCheckType = "flow_concurrent_render_count"
	// PolicyCheckDebugConcurrentRunCount asks whether the workspace may have
	// Value debug runs pending or running at once, counting the new one.
	// Deployment and trigger runs are not counted.
	PolicyCheckDebugConcurrentRunCount PolicyCheckType = "flow_debug_concurrent_run_count"
	// PolicyCheckDebugRunsPerHour asks whether the workspace may start Value
	// debug runs this hour, counting the new one.
	PolicyCheckDebugRunsPerHour PolicyCheckType = "flow_debug_runs_per_hour"
	// PolicyCheckDeploymentRunsPerDay asks whether the workspace may run its
	// deployments by hand Value times today, counting the new run.
	PolicyCheckDeploymentRunsPerDay PolicyCheckType = "flow_deployment_runs_per_day"
	// PolicyCheckEnabledTriggerCount asks whether the workspace may have Value
	// triggers enabled at once, API-driven and time-driven together, counting
	// the one being enabled.
	PolicyCheckEnabledTriggerCount PolicyCheckType = "flow_enabled_trigger_count"
	// PolicyCheckJobRunTimeSeconds asks whether a job may run for Value
	// seconds. CurrentLimit is the plan's limit in seconds, sent whether or not
	// Value is allowed.
	PolicyCheckJobRunTimeSeconds PolicyCheckType = "flow_job_run_time_seconds"
	// PolicyCheckTimeDrivenTriggerRunsPerDay asks whether the workspace's
	// time-driven triggers may run Value times today, counting the new run.
	PolicyCheckTimeDrivenTriggerRunsPerDay PolicyCheckType = "flow_time_driven_trigger_runs_per_day"
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
// policy service have none, and every constraint is allowed.
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
