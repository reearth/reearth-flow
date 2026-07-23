package main

import (
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestReadConfig_DiagnosticSubscriptionIDDefaultsEmpty pins that
// DiagnosticSubscriptionID defaults to "" (unlike its siblings): a
// defaulted value would crash-loop the whole subscriber (see config.go).
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

	// Sanity check: siblings still get real defaults (envconfig isn't just failing silently).
	assert.Equal(t, "flow-log-stream-main", conf.LogSubscriptionID)
	assert.Equal(t, "flow-node-status-main", conf.NodeSubscriptionID)
	assert.Equal(t, "flow-job-complete-main", conf.JobCompleteSubscriptionID)
	assert.Equal(t, "flow-user-facing-log-main", conf.UserFacingLogSubscriptionID)
}
