package postgres_test

import (
	"context"
	"regexp"
	"strings"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/pgtest"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// seedListQueryRows inserts enough rows (one target workspace plus noise from
// another) that the planner has a real choice between a sequential scan and
// an index scan for the paginated list queries below.
func seedListQueryRows(t *testing.T, ctx context.Context, exec func(ctx context.Context, sql string, args ...any) error) {
	t.Helper()

	stmts := []string{
		`INSERT INTO projects (id, workspace_id, workflow_id, name, description, is_archived, updated_at)
		 SELECT 'lqi_proj_' || i, CASE WHEN i % 10 = 0 THEN 'lqi_ws_other' ELSE 'lqi_ws_big' END, 'wf', 'P' || i, '', false, now() - (i || ' minutes')::interval
		 FROM generate_series(1, 500) i`,
		`INSERT INTO deployments (id, workspace_id, workflow_url, description, version, updated_at, is_head)
		 SELECT 'lqi_dep_' || i, CASE WHEN i % 10 = 0 THEN 'lqi_ws_other' ELSE 'lqi_ws_big' END, '', 'D' || i, 'v' || i, now() - (i || ' minutes')::interval, false
		 FROM generate_series(1, 500) i`,
		`INSERT INTO jobs (id, workspace_id, started_at, debug, status)
		 SELECT 'lqi_job_' || i, CASE WHEN i % 10 = 0 THEN 'lqi_ws_other' ELSE 'lqi_ws_big' END, now() - (i || ' minutes')::interval, (i % 50 = 0), 'COMPLETED'
		 FROM generate_series(1, 500) i`,
		`INSERT INTO triggers (id, workspace_id, deployment_id, description, event_source, created_at, updated_at)
		 SELECT 'lqi_trg_' || i, CASE WHEN i % 10 = 0 THEN 'lqi_ws_other' ELSE 'lqi_ws_big' END, 'lqi_dep_' || i, 'T' || i, 'API', now() - (i || ' minutes')::interval, now() - (i || ' minutes')::interval
		 FROM generate_series(1, 500) i`,
		`INSERT INTO assets (id, workspace_id, created_at, name, file_name)
		 SELECT 'lqi_asset_' || i, CASE WHEN i % 10 = 0 THEN 'lqi_ws_other' ELSE 'lqi_ws_big' END, now() - (i || ' minutes')::interval, 'A' || i, 'f' || i
		 FROM generate_series(1, 500) i`,
		`ANALYZE projects, deployments, jobs, triggers, assets`,
	}
	for _, s := range stmts {
		require.NoError(t, exec(ctx, s))
	}
}

// explainPlan runs EXPLAIN (without ANALYZE) and returns the plan text.
func explainPlan(t *testing.T, ctx context.Context, query func(ctx context.Context, sql string) (string, error), sql string) string {
	t.Helper()
	explain := "EXPLAIN " + sql
	plan, err := query(ctx, explain)
	require.NoError(t, err)
	return plan
}

// indexScanRegexp matches any scan-node variant (forward, backward,
// index-only, or bitmap) that names the given index, since the planner may
// pick any of them over a plain "Index Scan" while still using the index.
func indexScanRegexp(indexName string) *regexp.Regexp {
	name := regexp.QuoteMeta(indexName)
	return regexp.MustCompile(`(?:Index(?: Only)? Scan(?: Backward)? using|Bitmap Index Scan on) ` + name + `\b`)
}

// fullSortRegexp matches a plain "Sort" plan node but not "Incremental
// Sort", which still relies on the index for a presorted prefix.
var fullSortRegexp = regexp.MustCompile(`(?m)^\s*(?:->\s*)?(Incremental )?Sort\s+\(cost=`)

func hasFullSort(plan string) bool {
	for _, m := range fullSortRegexp.FindAllStringSubmatch(plan, -1) {
		if m[1] == "" {
			return true
		}
	}
	return false
}

func TestListQueries_UseCompositeIndexes(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()

	exec := func(ctx context.Context, sql string, args ...any) error {
		_, err := pool.Exec(ctx, sql, args...)
		return err
	}
	query := func(ctx context.Context, sql string) (string, error) {
		rows, err := pool.Query(ctx, sql)
		if err != nil {
			return "", err
		}
		defer rows.Close()
		var b strings.Builder
		for rows.Next() {
			var line string
			if err := rows.Scan(&line); err != nil {
				return "", err
			}
			b.WriteString(line)
			b.WriteString("\n")
		}
		return b.String(), rows.Err()
	}

	seedListQueryRows(t, ctx, exec)

	cases := []struct {
		name      string
		sql       string
		indexName string
	}{
		{
			name:      "projects default list",
			sql:       `SELECT * FROM projects WHERE workspace_id = 'lqi_ws_big' AND is_archived = false ORDER BY updated_at DESC LIMIT 20`,
			indexName: "projects_workspace_id_is_archived_updated_at_idx",
		},
		{
			name:      "deployments default list",
			sql:       `SELECT * FROM deployments WHERE workspace_id = 'lqi_ws_big' ORDER BY updated_at DESC LIMIT 20`,
			indexName: "deployments_workspace_id_updated_at_idx",
		},
		{
			name:      "jobs default list",
			sql:       `SELECT * FROM jobs WHERE workspace_id = 'lqi_ws_big' AND (debug IS NULL OR debug = false) ORDER BY started_at DESC LIMIT 20`,
			indexName: "jobs_workspace_id_started_at_idx",
		},
		{
			name:      "triggers default list",
			sql:       `SELECT * FROM triggers WHERE workspace_id = 'lqi_ws_big' ORDER BY updated_at DESC LIMIT 20`,
			indexName: "triggers_workspace_id_updated_at_idx",
		},
		{
			name:      "assets default list",
			sql:       `SELECT * FROM assets WHERE workspace_id = 'lqi_ws_big' ORDER BY created_at DESC LIMIT 20`,
			indexName: "assets_workspace_id_created_at_idx",
		},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			plan := explainPlan(t, ctx, query, tc.sql)
			assert.Regexp(t, indexScanRegexp(tc.indexName), plan, "plan should use %s:\n%s", tc.indexName, plan)
			assert.NotContains(t, plan, "Seq Scan", "plan should not fall back to a sequential scan:\n%s", plan)
			assert.False(t, hasFullSort(plan), "index should satisfy ORDER BY without a full sort step:\n%s", plan)
		})
	}
}

// TestListQueries_CompositeIndexesExist pins the exact column order the
// migration must create: it fails immediately (rather than via planner
// heuristics) if an index is missing or its column order doesn't match the
// query's WHERE + ORDER BY shape.
func TestListQueries_CompositeIndexesExist(t *testing.T) {
	pool := pgtest.Connect(t)(t)
	ctx := context.Background()

	cases := []struct {
		table   string
		index   string
		wantDef string
	}{
		{"projects", "projects_workspace_id_is_archived_updated_at_idx",
			`CREATE INDEX projects_workspace_id_is_archived_updated_at_idx ON public.projects USING btree (workspace_id, is_archived, updated_at DESC)`},
		{"deployments", "deployments_workspace_id_updated_at_idx",
			`CREATE INDEX deployments_workspace_id_updated_at_idx ON public.deployments USING btree (workspace_id, updated_at DESC)`},
		{"jobs", "jobs_workspace_id_started_at_idx",
			`CREATE INDEX jobs_workspace_id_started_at_idx ON public.jobs USING btree (workspace_id, started_at DESC)`},
		{"triggers", "triggers_workspace_id_updated_at_idx",
			`CREATE INDEX triggers_workspace_id_updated_at_idx ON public.triggers USING btree (workspace_id, updated_at DESC)`},
		{"assets", "assets_workspace_id_created_at_idx",
			`CREATE INDEX assets_workspace_id_created_at_idx ON public.assets USING btree (workspace_id, created_at DESC)`},
	}

	for _, tc := range cases {
		t.Run(tc.index, func(t *testing.T) {
			var indexdef string
			row := pool.QueryRow(ctx, `SELECT indexdef FROM pg_indexes WHERE tablename = $1 AND indexname = $2`, tc.table, tc.index)
			require.NoError(t, row.Scan(&indexdef), "index %s must exist on table %s", tc.index, tc.table)
			assert.Equal(t, tc.wantDef, indexdef)
		})
	}
}
