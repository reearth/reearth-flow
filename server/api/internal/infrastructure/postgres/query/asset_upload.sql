-- name: GetAssetUpload :one
SELECT * FROM asset_uploads WHERE uuid = $1;

-- name: UpsertAssetUpload :exec
INSERT INTO asset_uploads (
  uuid, workspace_id, file_name, content_type, content_encoding,
  content_length, expires_at
) VALUES ($1,$2,$3,$4,$5,$6,$7)
ON CONFLICT (uuid) DO UPDATE SET
  workspace_id     = EXCLUDED.workspace_id,
  file_name        = EXCLUDED.file_name,
  content_type     = EXCLUDED.content_type,
  content_encoding = EXCLUDED.content_encoding,
  content_length   = EXCLUDED.content_length,
  expires_at       = EXCLUDED.expires_at;
