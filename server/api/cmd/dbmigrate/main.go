// Command dbmigrate replicates flow-owned data from MongoDB to PostgreSQL.
//
// It streams every document in each Mongo collection, decodes it with the same
// mongodoc decoder the Mongo repositories use, and upserts it through the same
// Postgres adapter Save the API uses. Both sides are the production-tested
// read/write paths, so the ETL inherits their field mappings exactly and is
// idempotent (Save is an upsert, so re-runs are safe).
//
// Scope: the 12 flow-owned data repos. Account repos (user/workspace/role/
// permittable) stay on Mongo and are not copied. Config (migration bookkeeping)
// and AuthRequest (transient OAuth state) are intentionally skipped.
//
// Usage:
//
//	REEARTH_FLOW_DB="mongodb+srv://…"  (source Mongo)
//	REEARTH_FLOW_DB_PG="postgres://…"  (target Postgres)
//	go run ./cmd/dbmigrate [-apply-schema] [-db reearth-flow]
package main

import (
	"context"
	"flag"
	"log"
	"os"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/mongo/mongodoc"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/db"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/pgxx"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
)

// migrate streams a Mongo collection through its mongodoc decoder into the
// Postgres adapter. D is the mongodoc document type, M the domain model.
func migrate[D any, M any](
	ctx context.Context,
	db *mongo.Database,
	coll string,
	toModel func(*D) (M, error),
	save func(context.Context, M) error,
) (ok, fail int) {
	cur, err := db.Collection(coll).Find(ctx, bson.D{})
	if err != nil {
		// Count as a failure so a fully-skipped collection forces a non-zero exit
		// instead of silently reporting success.
		log.Printf("  %-16s find error: %v", coll, err)
		return 0, 1
	}
	defer cur.Close(ctx)

	for cur.Next(ctx) {
		var d D
		if err := cur.Decode(&d); err != nil {
			log.Printf("  %-16s decode error: %v", coll, err)
			fail++
			continue
		}
		m, err := toModel(&d)
		if err != nil {
			log.Printf("  %-16s model error: %v", coll, err)
			fail++
			continue
		}
		if err := save(ctx, m); err != nil {
			log.Printf("  %-16s save error: %v", coll, err)
			fail++
			continue
		}
		ok++
	}
	if err := cur.Err(); err != nil {
		// A mid-stream cursor error truncates the collection; count it so the
		// partial migration forces a non-zero exit rather than passing silently.
		log.Printf("  %-16s cursor error (partial): %v", coll, err)
		fail++
	}
	log.Printf("  %-16s migrated=%-7d failed=%d", coll, ok, fail)
	return ok, fail
}

func main() {
	log.SetFlags(0)

	dbName := flag.String("db", envOr("REEARTH_FLOW_DB_NAME", "reearth-flow"), "source Mongo database name")
	verify := flag.Bool("verify", false, "read every replicated row back through the Postgres adapters (target only; no Mongo)")
	applySchema := flag.Bool("apply-schema", false, "apply the embedded Atlas migrations to the target before replicating (fresh instance only)")
	flag.Parse()

	pgURI := os.Getenv("REEARTH_FLOW_DB_PG")
	if pgURI == "" {
		log.Fatal("set REEARTH_FLOW_DB_PG (target Postgres URI)")
	}

	ctx := context.Background()

	pool, err := pgxpool.New(ctx, pgURI)
	if err != nil {
		log.Fatalf("postgres connect: %v", err)
	}
	defer pool.Close()
	if err := pool.Ping(ctx); err != nil {
		log.Fatalf("postgres ping: %v", err)
	}

	if *applySchema {
		log.Println("applying embedded schema to target Postgres")
		if err := db.Apply(ctx, pool); err != nil {
			log.Fatalf("apply schema: %v", err)
		}
	}

	c := pgxx.NewClient(pool)

	if *verify {
		runVerify(ctx, pool, c)
		return
	}

	mongoURI := os.Getenv("REEARTH_FLOW_DB")
	if mongoURI == "" {
		log.Fatal("set REEARTH_FLOW_DB (source Mongo URI)")
	}
	mc, err := mongo.Connect(ctx, options.Client().ApplyURI(mongoURI))
	if err != nil {
		log.Fatalf("mongo connect: %v", err)
	}
	defer func() { _ = mc.Disconnect(ctx) }()
	if err := mc.Ping(ctx, nil); err != nil {
		log.Fatalf("mongo ping: %v", err)
	}
	db := mc.Database(*dbName)

	log.Printf("ETL: Mongo %q -> Postgres (flow-owned repos)", *dbName)
	start := time.Now()

	var totalOK, totalFail int
	acc := func(ok, fail int) { totalOK += ok; totalFail += fail }

	acc(migrate(ctx, db, "project", (*mongodoc.ProjectDocument).Model, postgres.NewProject(c).Save))
	acc(migrate(ctx, db, "workflow", (*mongodoc.WorkflowDocument).Model, postgres.NewWorkflow(c).Save))
	acc(migrate(ctx, db, "parameter", (*mongodoc.ParameterDocument).Model, postgres.NewParameter(c).Save))
	acc(migrate(ctx, db, "deployment", (*mongodoc.DeploymentDocument).Model, postgres.NewDeployment(c).Save))
	acc(migrate(ctx, db, "trigger", (*mongodoc.TriggerDocument).Model, postgres.NewTrigger(c).Save))
	acc(migrate(ctx, db, "job", (*mongodoc.JobDocument).Model, postgres.NewJob(c).Save))
	acc(migrate(ctx, db, "edgeExecutions", (*mongodoc.EdgeExecutionDocument).Model, postgres.NewEdgeExecution(c).Save))
	acc(migrate(ctx, db, "nodeExecutions", (*mongodoc.NodeExecutionDocument).Model, postgres.NewNodeExecution(c).Save))
	acc(migrate(ctx, db, "projectAccess", (*mongodoc.ProjectAccessDocument).Model, postgres.NewProjectAccess(c).Save))
	acc(migrate(ctx, db, "asset", (*mongodoc.AssetDocument).Model, postgres.NewAsset(c).Save))
	acc(migrate(ctx, db, "asset_upload", (*mongodoc.AssetUploadDocument).Model, postgres.NewAssetUpload(c).Save))
	acc(migrate(ctx, db, "worker_config", (*mongodoc.WorkerConfigDocument).Model, postgres.NewWorkerConfig(c).Save))

	log.Printf("done in %s: migrated=%d failed=%d", time.Since(start).Round(time.Millisecond), totalOK, totalFail)
	if totalFail > 0 {
		os.Exit(1)
	}
}

// runVerify reads every replicated row back through the Postgres adapters,
// confirming the pg→domain decode path works on real data (the migrate side
// only proves the domain→pg write). Covers the entities that expose FindByIDs.
func runVerify(ctx context.Context, pool *pgxpool.Pool, c *pgxx.Client) {
	log.Println("VERIFY: reading replicated rows back through the Postgres adapters")
	var bad int
	bad += verifyOne(ctx, pool, "projects", func(ids []string) (int, error) {
		l, err := id.ProjectIDListFrom(ids)
		if err != nil {
			return 0, err
		}
		r, err := postgres.NewProject(c).FindByIDs(ctx, l)
		return len(r), err
	})
	bad += verifyOne(ctx, pool, "deployments", func(ids []string) (int, error) {
		l, err := id.DeploymentIDListFrom(ids)
		if err != nil {
			return 0, err
		}
		r, err := postgres.NewDeployment(c).FindByIDs(ctx, l)
		return len(r), err
	})
	bad += verifyOne(ctx, pool, "triggers", func(ids []string) (int, error) {
		l, err := id.TriggerIDListFrom(ids)
		if err != nil {
			return 0, err
		}
		r, err := postgres.NewTrigger(c).FindByIDs(ctx, l)
		return len(r), err
	})
	bad += verifyOne(ctx, pool, "jobs", func(ids []string) (int, error) {
		l, err := id.JobIDListFrom(ids)
		if err != nil {
			return 0, err
		}
		r, err := postgres.NewJob(c).FindByIDs(ctx, l)
		return len(r), err
	})
	bad += verifyOne(ctx, pool, "assets", func(ids []string) (int, error) {
		l, err := id.AssetIDListFrom(ids)
		if err != nil {
			return 0, err
		}
		r, err := postgres.NewAsset(c).FindByIDs(ctx, l)
		return len(r), err
	})
	bad += verifyOne(ctx, pool, "parameters", func(ids []string) (int, error) {
		l, err := id.ParameterIDListFrom(ids)
		if err != nil {
			return 0, err
		}
		r, err := postgres.NewParameter(c).FindByIDs(ctx, l)
		if r == nil {
			return 0, err
		}
		return len(*r), err
	})
	if bad > 0 {
		log.Printf("VERIFY: %d entity(ies) had read-back issues", bad)
		os.Exit(1)
	}
	log.Println("VERIFY: all rows read back and decoded cleanly")
}

func verifyOne(ctx context.Context, pool *pgxpool.Pool, table string, readBack func([]string) (int, error)) int {
	rows, err := pool.Query(ctx, "SELECT id FROM "+table)
	if err != nil {
		log.Printf("  %-16s id query error: %v", table, err)
		return 1
	}
	var ids []string
	for rows.Next() {
		var s string
		if err := rows.Scan(&s); err != nil {
			rows.Close()
			log.Printf("  %-16s scan error: %v", table, err)
			return 1
		}
		ids = append(ids, s)
	}
	rows.Close()
	if err := rows.Err(); err != nil {
		log.Printf("  %-16s rows error: %v", table, err)
		return 1
	}
	got, err := readBack(ids)
	if err != nil {
		log.Printf("  %-16s read-back decode error: %v", table, err)
		return 1
	}
	status := "ok"
	bad := 0
	if got != len(ids) {
		status = "MISMATCH (some rows failed to decode)"
		bad = 1
	}
	log.Printf("  %-16s rows=%-7d decoded=%-7d %s", table, len(ids), got, status)
	return bad
}

func envOr(key, def string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return def
}
