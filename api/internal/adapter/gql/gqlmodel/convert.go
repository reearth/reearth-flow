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
		TotalCount:  int(p.TotalCount),
		CurrentPage: currentPage,
		TotalPages:  totalPages,
	}
}

func ToPagination(pagination *Pagination) *usecasex.Pagination {
	if pagination == nil {
		return nil
	}

	// Page-based pagination
	if pagination.Page != nil && pagination.PageSize != nil {
		return &usecasex.Pagination{
			Offset: &usecasex.OffsetPagination{
				Offset: int64((*pagination.Page - 1) * *pagination.PageSize),
				Limit:  int64(*pagination.PageSize),
			},
		}
	}

	return nil
}

func ToPageBasedPagination(pagination PageBasedPagination) *usecasex.Pagination {
	return &usecasex.Pagination{
		Offset: &usecasex.OffsetPagination{
			Offset: int64((pagination.Page - 1) * pagination.PageSize),
			Limit:  int64(pagination.PageSize),
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
		TotalCount: int64(p.TotalCount),
	}
}
