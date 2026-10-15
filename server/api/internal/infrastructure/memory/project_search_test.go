package memory

import (
	"testing"

	"github.com/reearth/reearth-flow/api/internal/testutil/repotest"
)

func TestProject_Search(t *testing.T) {
	repotest.ProjectSearch(t, NewProject())
}
