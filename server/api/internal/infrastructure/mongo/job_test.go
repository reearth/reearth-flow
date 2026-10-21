package mongo

import (
	"context"
	"os"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/reearth/reearth-flow/api/internal/testutil/repotest"
	"github.com/reearth/reearthx/mongox"
	"github.com/reearth/reearthx/usecasex"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
)

// connectJobDB opts these tests into REEARTH_FLOW_DB directly, like
// connectProjectDB, and creates the job indexes.
func connectJobDB(t *testing.T) *Job {
	t.Helper()
	r, _ := connectJobClient(t)
	return r
}

// connectJobTxDB is connectJobDB with transactions, wired as New wires them
// when they are on. Transactions need a replica set, so it skips without one.
func connectJobTxDB(t *testing.T) (*Job, usecasex.Transactor) {
	t.Helper()
	r, client := connectJobClient(t)
	var hello struct {
		SetName string `bson:"setName"`
	}
	require.NoError(t, client.Database().RunCommand(context.Background(), bson.D{{Key: "hello", Value: 1}}).Decode(&hello))
	if hello.SetName == "" {
		t.Skip("REEARTH_FLOW_DB is not a replica set, so it has no transactions")
	}
	return r, usecasex.NewTransactor(client.WithTransaction().Transaction(), 2)
}

func connectJobClient(t *testing.T) (*Job, *mongox.Client) {
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
	client := mongox.NewClientWithDatabase(db)
	r := NewJob(client).(*Job)
	require.NoError(t, r.Init(ctx))
	return r, client
}

func TestJob_CountRunningByWorkspace(t *testing.T) {
	repotest.JobCountRunning(t, connectJobDB(t))
}

func TestJob_SaveIfRunningBelow(t *testing.T) {
	repotest.JobSaveIfRunningBelow(t, connectJobDB(t))
}

func TestJob_SaveIfRunningBelow_InTransaction(t *testing.T) {
	r, tr := connectJobTxDB(t)
	repotest.JobSaveIfRunningBelowInTx(t, r, tr)
}

func TestJob_InitTwice(t *testing.T) {
	r := connectJobDB(t)
	assert.NoError(t, r.Init(context.Background()), "the admission collection already exists")
}
