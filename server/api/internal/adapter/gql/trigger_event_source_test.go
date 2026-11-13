package gql

import (
	"testing"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestTriggerEventSource_CMS(t *testing.T) {
	t.Parallel()

	cms := gqlmodel.EventSourceTypeCms
	source, interval, err := triggerEventSource(&cms, nil)

	require.NoError(t, err)
	assert.Equal(t, trigger.EventSourceTypeCMS, source)
	assert.Empty(t, interval)
}
