package main

import (
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestReadConfig_DiagnosticSubscriptionIDDefaultsEmpty pins the deploy-gate
// fix: DiagnosticSubscriptionID must default to "" (unlike its sibling
// *SubscriptionID fields, which all default to a real name), so that
// `conf.DiagnosticSubscriptionID != ""` in main.go stays false until an
// operator explicitly provisions and wires the subscription. Defaulting it
// to a name would make the subscriber try to open a listener for a
// subscription that was never provisioned in that environment; since a
// listener error cancels the subscriber's root context, that crash-loops
// the whole process, not just diagnostics ingestion.
func TestReadConfig_DiagnosticSubscriptionIDDefaultsEmpty(t *testing.T) {
	for _, key := range []string{
		"REEARTH_FLOW_SUBSCRIBER_DIAGNOSTIC_SUBSCRIPTION_ID",
		"REEARTH_FLOW_SUBSCRIBER_LOG_SUBSCRIPTION_ID",
		"REEARTH_FLOW_SUBSCRIBER_NODE_STATUS_SUBSCRIPTION_ID",
		"REEARTH_FLOW_SUBSCRIBER_JOB_COMPLETE_SUBSCRIPTION_ID",
		"REEARTH_FLOW_SUBSCRIBER_USER_FACING_LOG_SUBSCRIPTION_ID",
	} {
		t.Setenv(key, "")
		require.NoError(t, os.Unsetenv(key))
	}

	conf, err := ReadConfig(false)
	require.NoError(t, err)

	assert.Equal(t, "", conf.DiagnosticSubscriptionID,
		"DiagnosticSubscriptionID must default to empty so the subscriber "+
			"skips starting the diagnostic listener until explicitly configured")

	// Sanity check: the sibling subscription IDs still get their real
	// defaults, so this isn't a case of envconfig silently failing to
	// populate defaults at all.
	assert.Equal(t, "flow-log-stream-main", conf.LogSubscriptionID)
	assert.Equal(t, "flow-node-status-main", conf.NodeSubscriptionID)
	assert.Equal(t, "flow-job-complete-main", conf.JobCompleteSubscriptionID)
	assert.Equal(t, "flow-user-facing-log-main", conf.UserFacingLogSubscriptionID)
}
