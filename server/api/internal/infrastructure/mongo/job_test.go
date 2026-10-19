package mongo

import (
	"context"
	"os"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/reearth/reearth-flow/api/internal/testutil/repotest"
	"github.com/reearth/reearthx/mongox"
	"github.com/stretchr/testify/require"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
)

// connectJobDB opts these tests into REEARTH_FLOW_DB directly, like
// connectProjectDB, and creates the job indexes.
func connectJobDB(t *testing.T) *Job {
	t.Helper()
	uri := os.Getenv("REEARTH_FLOW_DB")
	if uri == "" {
		t.Skip("REEARTH_FLOW_DB is not set")
	}
	ctx := context.Background()
	c, err := mongo.Connect(ctx, options.Client().ApplyURI(uri).SetConnectTimeout(10*time.Second))
	require.NoError(t, err)
	db := c.Database("test_" + uuid.NewString())
	t.Cleanup(func() {
		_ = db.Drop(context.Background())
		_ = c.Disconnect(context.Background())
	})
	r := NewJob(mongox.NewClientWithDatabase(db)).(*Job)
	require.NoError(t, r.Init(ctx))
	return r
}

func TestJob_CountRunningByWorkspace(t *testing.T) {
	repotest.JobCountRunning(t, connectJobDB(t))
}

func TestJob_SaveIfRunningBelow(t *testing.T) {
	repotest.JobSaveIfRunningBelow(t, connectJobDB(t))
}
