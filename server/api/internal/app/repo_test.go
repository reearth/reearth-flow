package app

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/stretchr/testify/assert"
)

// Without a policy service there is no checker, so the plan limits skip the
// counting they would need to enforce a limit.
func TestInitPolicyChecker_PermissiveIsNone(t *testing.T) {
	for _, typ := range []string{"", "permissive"} {
		assert.Nil(t, initPolicyChecker(context.Background(), &config.Config{PolicyChecker_Type: typ}), typ)
	}
}
