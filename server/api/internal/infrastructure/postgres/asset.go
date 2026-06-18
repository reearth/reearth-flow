package postgres

import (
	"context"
	"fmt"
	"strings"

	"github.com/jackc/pgx/v5"
	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/gen"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/rerror"
)

type Asset struct {
	c *pgxx.Client
	f repo.WorkspaceFilter
}

var _ repo.Asset = (*Asset)(nil)

func NewAsset(c *pgxx.Client) *Asset {
	return &Asset{c: c}
}

func (r *Asset) Filtered(f repo.WorkspaceFilter) repo.Asset {
	return &Asset{c: r.c, f: r.f.Merge(f)}
}

func (r *Asset) q(ctx context.Context) *gen.Queries {
	return gen.New(r.c.DB(ctx))
}

func (r *Asset) FindByID(ctx context.Context, aid id.AssetID) (*asset.Asset, error) {
	row, err := r.q(ctx).GetAsset(ctx, aid.String())
	if err != nil {
		return nil, pgxx.MapError(err)
	}
	a, err := assetFromRow(row)
	if err != nil {
		return nil, err
	}
	if !r.f.CanRead(a.Workspace()) {
		return nil, rerror.ErrNotFound
	}
	return a, nil
}

func (r *Asset) FindByIDs(ctx context.Context, ids id.AssetIDList) ([]*asset.Asset, error) {
	rows, err := r.q(ctx).ListAssetsByIDs(ctx, ids.Strings())
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	as := make([]*asset.Asset, 0, len(rows))
	for _, row := range rows {
		a, err := assetFromRow(row)
		if err != nil {
			return nil, err
		}
		if r.f.CanRead(a.Workspace()) {
			as = append(as, a)
		}
	}
	return pgxx.OrderByIDs(ids.Strings(), as, func(a *asset.Asset) string { return a.ID().String() }), nil
}

var assetOrderByColumns = map[string]string{
	"name":      "name",
	"id":        "id",
	"size":      "size",
	"createdAt": "created_at",
	"date":      "created_at",
}

func (r *Asset) FindByWorkspace(
	ctx context.Context,
	wid accountsid.WorkspaceID,
	filter repo.AssetFilter,
) ([]*asset.Asset, *interfaces.PageBasedInfo, error) {
	if !r.f.CanRead(wid) {
		return nil, interfaces.NewPageBasedInfo(0, 1, 1), nil
	}

	where := []string{"workspace_id = $1"}
	args := []any{wid.String()}

	if filter.Keyword != nil && *filter.Keyword != "" {
		args = append(args, "%"+*filter.Keyword+"%")
		where = append(where, fmt.Sprintf("name ILIKE $%d", len(args)))
	}

	whereSQL := "WHERE " + strings.Join(where, " AND ")
	db := r.c.DB(ctx)

	// Determine sort column and direction
	orderCol := "created_at"
	dir := "DESC"
	if filter.Sort != nil {
		if c, ok := assetOrderByColumns[filter.Sort.Key]; ok {
			orderCol = c
		}
		dir = "ASC"
	}

	if filter.Pagination != nil && filter.Pagination.Page != nil {
		p := filter.Pagination.Page
		if p.OrderBy != nil {
			if c, ok := assetOrderByColumns[*p.OrderBy]; ok {
				orderCol = c
			}
			dir = "ASC"
			if p.OrderDir != nil && strings.EqualFold(*p.OrderDir, "DESC") {
				dir = "DESC"
			}
		}
		listSQL := fmt.Sprintf("SELECT * FROM assets %s ORDER BY %s %s", whereSQL, orderCol, dir)
		rows, total, err := pgxx.CountAndList(ctx, db,
			"SELECT count(*) FROM assets "+whereSQL, listSQL, args,
			int64(p.PageSize), int64((p.Page-1)*p.PageSize),
			pgx.RowToStructByPos[gen.Asset])
		if err != nil {
			return nil, nil, err
		}
		as, err := assetsFromRows(rows)
		if err != nil {
			return nil, nil, err
		}
		return as, interfaces.NewPageBasedInfo(total, p.Page, p.PageSize), nil
	}

	rows, err := pgxx.List(ctx, db,
		fmt.Sprintf("SELECT * FROM assets %s ORDER BY %s %s", whereSQL, orderCol, dir), args,
		pgx.RowToStructByPos[gen.Asset])
	if err != nil {
		return nil, nil, err
	}
	as, err := assetsFromRows(rows)
	if err != nil {
		return nil, nil, err
	}
	return as, interfaces.NewPageBasedInfo(int64(len(as)), 1, len(as)), nil
}

func (r *Asset) TotalSizeByWorkspace(ctx context.Context, wid accountsid.WorkspaceID) (uint64, error) {
	if !r.f.CanRead(wid) {
		return 0, repo.ErrOperationDenied
	}
	var total int64
	err := r.c.DB(ctx).QueryRow(ctx,
		`SELECT COALESCE(SUM(size), 0) FROM assets WHERE workspace_id = $1`,
		wid.String(),
	).Scan(&total)
	if err != nil {
		return 0, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return uint64(total), nil
}

func (r *Asset) Save(ctx context.Context, a *asset.Asset) error {
	if !r.f.CanWrite(a.Workspace()) {
		return repo.ErrOperationDenied
	}
	if err := r.q(ctx).UpsertAsset(ctx, assetToParams(a)); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func (r *Asset) Delete(ctx context.Context, aid id.AssetID) error {
	exec := r.c.DB(ctx)
	if r.f.Writable == nil {
		if _, err := exec.Exec(ctx, `DELETE FROM assets WHERE id = $1`, aid.String()); err != nil {
			return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
		}
		return nil
	}
	if _, err := exec.Exec(ctx,
		`DELETE FROM assets WHERE id = $1 AND workspace_id = ANY($2::text[])`,
		aid.String(), r.f.Writable.Strings(),
	); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func assetsFromRows(rows []gen.Asset) ([]*asset.Asset, error) {
	res := make([]*asset.Asset, 0, len(rows))
	for _, row := range rows {
		a, err := assetFromRow(row)
		if err != nil {
			return nil, err
		}
		res = append(res, a)
	}
	return res, nil
}

func assetToParams(a *asset.Asset) gen.UpsertAssetParams {
	p := gen.UpsertAssetParams{
		ID:          a.ID().String(),
		WorkspaceID: a.Workspace().String(),
		CreatedAt:   a.CreatedAt(),
		Name:        a.Name(),
		FileName:    a.FileName(),
		Size:        int64(a.Size()), //nolint:gosec
		Url:         a.URL(),
		ContentType: a.ContentType(),
		Uuid:        a.UUID(),
		FlatFiles:   a.FlatFiles(),
		Public:      a.Public(),
	}

	// project_id: only store if it's a real project (not the workspace-only sentinel)
	if pid := a.Project(); !pid.IsNil() {
		s := pid.String()
		p.ProjectID = &s
	}

	if u := a.User(); u != nil {
		s := u.String()
		p.UserID = &s
	}

	if iid := a.Integration(); iid != nil {
		s := iid.String()
		p.IntegrationID = &s
	}

	if tid := a.Thread(); tid != nil {
		s := tid.String()
		p.ThreadID = &s
	}

	if aes := a.ArchiveExtractionStatus(); aes != nil {
		s := string(*aes)
		p.ArchiveExtractionStatus = &s
	}

	return p
}

func assetFromRow(row gen.Asset) (*asset.Asset, error) {
	aid, err := id.AssetIDFrom(row.ID)
	if err != nil {
		return nil, err
	}
	wid, err := accountsid.WorkspaceIDFrom(row.WorkspaceID)
	if err != nil {
		return nil, err
	}

	b := asset.New().
		ID(aid).
		Workspace(wid).
		CreatedAt(row.CreatedAt).
		Name(row.Name).
		FileName(row.FileName).
		Size(uint64(row.Size)). //nolint:gosec
		URL(row.Url).
		ContentType(row.ContentType).
		UUID(row.Uuid).
		FlatFiles(row.FlatFiles).
		Public(row.Public)

	if row.ProjectID != nil {
		pid, err := id.ProjectIDFrom(*row.ProjectID)
		if err != nil {
			return nil, err
		}
		b = b.Project(pid)
	}

	if row.UserID != nil {
		uid, err := accountsid.UserIDFrom(*row.UserID)
		if err != nil {
			return nil, err
		}
		b = b.CreatedByUser(uid)
	}

	if row.IntegrationID != nil {
		iid, err := id.IntegrationIDFrom(*row.IntegrationID)
		if err != nil {
			return nil, err
		}
		b = b.CreatedByIntegration(&iid)
	}

	if row.ThreadID != nil {
		tid, err := id.ThreadIDFrom(*row.ThreadID)
		if err != nil {
			return nil, err
		}
		b = b.Thread(&tid)
	}

	if row.ArchiveExtractionStatus != nil {
		aes, ok := asset.ArchiveExtractionStatusFrom(*row.ArchiveExtractionStatus)
		if ok {
			b = b.ArchiveExtractionStatus(aes)
		}
	}

	return b.Build()
}
