package gql

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/samber/lo"
	"github.com/stretchr/testify/assert"
)

// projectSnapshot serves two addressing modes through one field, which the schema
// cannot constrain to exactly one. These guards are that constraint, so they are
// asserted here rather than left to the caller.
//
// Both reject before any usecase call, so a nil resolver is enough: reaching the
// websocket client would panic and fail the test.
func TestProjectSnapshot_RejectsAmbiguousArguments(t *testing.T) {
	r := &queryResolver{}

	t.Run("both arguments", func(t *testing.T) {
		_, err := r.ProjectSnapshot(context.Background(), gqlmodel.ID("p1"), lo.ToPtr(1), lo.ToPtr(2))
		assert.ErrorContains(t, err, "not both")
	})

	t.Run("neither argument", func(t *testing.T) {
		_, err := r.ProjectSnapshot(context.Background(), gqlmodel.ID("p1"), nil, nil)
		assert.ErrorContains(t, err, "either version or snapshotNumber")
	})
}
