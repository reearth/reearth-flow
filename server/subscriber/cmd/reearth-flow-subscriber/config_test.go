package main

import (
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// Unlike its siblings, a default here would crash-loop the whole subscriber.
func TestReadConfig_DiagnosticSubscriptionIDDefaultsEmpty(t *testing.T) {
	for _, key := range []string{
		"REEARTH_FLOW_SUBSCRIBER_DIAGNOSTIC_SUBSCRIPTION_ID",
		"REEARTH_FLOW_SUBSCRIBER_LOG_SUBSCRIPTION_ID",
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

	// Sanity check that envconfig is not silently failing.
	assert.Equal(t, "flow-log-stream-main", conf.LogSubscriptionID)
	assert.Equal(t, "flow-job-complete-main", conf.JobCompleteSubscriptionID)
	assert.Equal(t, "flow-user-facing-log-main", conf.UserFacingLogSubscriptionID)
}
