-- name: GetAsset :one
SELECT * FROM assets WHERE id = $1;

-- name: ListAssetsByIDs :many
SELECT * FROM assets WHERE id = ANY($1::text[]);

-- name: UpsertAsset :exec
INSERT INTO assets (
  id, workspace_id, created_at, name, file_name, size, url, content_type,
  uuid, flat_files, public, project_id, user_id, integration_id,
  thread_id, archive_extraction_status
) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
ON CONFLICT (id) DO UPDATE SET
  workspace_id               = EXCLUDED.workspace_id,
  created_at                 = EXCLUDED.created_at,
  name                       = EXCLUDED.name,
  file_name                  = EXCLUDED.file_name,
  size                       = EXCLUDED.size,
  url                        = EXCLUDED.url,
  content_type               = EXCLUDED.content_type,
  uuid                       = EXCLUDED.uuid,
  flat_files                 = EXCLUDED.flat_files,
  public                     = EXCLUDED.public,
  project_id                 = EXCLUDED.project_id,
  user_id                    = EXCLUDED.user_id,
  integration_id             = EXCLUDED.integration_id,
  thread_id                  = EXCLUDED.thread_id,
  archive_extraction_status  = EXCLUDED.archive_extraction_status;

-- name: DeleteAsset :exec
DELETE FROM assets WHERE id = $1;
