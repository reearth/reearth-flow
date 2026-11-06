package trigger

import "time"

// RefusalCode names why a scheduled run was refused.
type RefusalCode string

const (
	// RefusalNotFound: the trigger or its deployment no longer exists. Only a
	// trigger that exists records a refusal, so there it is the deployment.
	RefusalNotFound             RefusalCode = "not_found"
	RefusalTriggerDisabled      RefusalCode = "trigger_disabled"
	RefusalTriggerNotTimeDriven RefusalCode = "trigger_not_time_driven"
	// RefusalInvalidVariables: a required variable has no value, or a value
	// does not fit its variable.
	RefusalInvalidVariables RefusalCode = "invalid_variables"
	RefusalNotAllowed       RefusalCode = "operation_denied"
	// RefusalConcurrentJobLimit: the workspace already runs as many jobs as its
	// plan allows.
	RefusalConcurrentJobLimit RefusalCode = "concurrent_job_limit"
	// RefusalServerError: the server could not start the run.
	RefusalServerError RefusalCode = "server_error"
)

// Refusal is the latest refusal of a trigger's scheduled run that no started
// run has followed yet.
type Refusal struct {
	At time.Time
	// RetryUntil is when the scheduler stops retrying the run. It is nil for a
	// run that is skipped, as no retry can succeed.
	RetryUntil *time.Time
	Code       RefusalCode
}
