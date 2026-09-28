# The coordination schema is authored in ../db (ws-schema.hcl) with Atlas and
# vendored here, because the production image builds with context: server/websocket-go
# and cannot reach outside this module.

.PHONY: sync-migrations check-migrations

# Refresh the committed copy after `atlas migrate diff --env ws` in ../db.
sync-migrations:
	rm -f internal/pg/migrations/*.sql
	cp ../db/ws-migrations/*.sql internal/pg/migrations/

# Fails when the copy has drifted from the source of truth. Run by CI.
check-migrations:
	@diff -r --exclude=atlas.sum ../db/ws-migrations internal/pg/migrations \
	  || { echo "internal/pg/migrations has drifted from server/db/ws-migrations; run 'make sync-migrations'"; exit 1; }
	@echo "migrations in sync with server/db/ws-migrations"
