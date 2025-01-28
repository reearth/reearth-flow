package gqlmodel

import (
	"io"

	"github.com/99designs/gqlgen/graphql"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearthx/usecasex"
	"github.com/samber/lo"
)

func FromFile(f *graphql.Upload) *file.File {
	if f == nil {
		return nil
	}
	return &file.File{
		Content:     io.NopCloser(f.File),
		Path:        f.Filename,
		Size:        f.Size,
		ContentType: f.ContentType,
	}
}

func ToPageInfo(p *usecasex.PageInfo) *PageInfo {
	if p == nil {
		return &PageInfo{}
	}

	// Check if this is a page-based info
	var currentPage, totalPages *int
	if pbi, ok := any(p).(*interfaces.PageBasedInfo); ok {
		cp := pbi.CurrentPage
		tp := pbi.TotalPages
		currentPage = &cp
		totalPages = &tp
	}

	return &PageInfo{
		StartCursor:     p.StartCursor,
		EndCursor:       p.EndCursor,
		HasNextPage:     p.HasNextPage,
		HasPreviousPage: p.HasPreviousPage,
		TotalCount:      int(p.TotalCount),
		CurrentPage:     currentPage,
		TotalPages:      totalPages,
	}
}

func ToPagination(pagination *Pagination) *usecasex.Pagination {
	if pagination == nil {
		return nil
	}

	// Cursor-based pagination
	return usecasex.CursorPagination{
		Before: pagination.Before,
		After:  pagination.After,
		First:  intToInt64(pagination.First),
		Last:   intToInt64(pagination.Last),
	}.Wrap()
}

func ToPageBasedPagination(pagination PageBasedPagination) *interfaces.PaginationParam {
	return &interfaces.PaginationParam{
		Page: &interfaces.PageBasedPaginationParam{
			Page:     pagination.Page,
			PageSize: pagination.PageSize,
			OrderBy:  pagination.OrderBy,
			OrderDir: OrderDirectionToString(pagination.OrderDir),
		},
	}
}

func OrderDirectionToString(dir *OrderDirection) *string {
	if dir == nil {
		return nil
	}
	s := string(*dir)
	return &s
}

func intToInt64(i *int) *int64 {
	if i == nil {
		return nil
	}
	return lo.ToPtr(int64(*i))
}

func FromPageInfo(p *PageInfo) *usecasex.PageInfo {
	if p == nil {
		return &usecasex.PageInfo{}
	}
	return &usecasex.PageInfo{
		StartCursor:     p.StartCursor,
		EndCursor:       p.EndCursor,
		HasNextPage:     p.HasNextPage,
		HasPreviousPage: p.HasPreviousPage,
		TotalCount:      int64(p.TotalCount),
	}
}
