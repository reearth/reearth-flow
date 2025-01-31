package gqlmodel

import (
	"io"

	"github.com/99designs/gqlgen/graphql"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearthx/usecasex"
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

func ToPageInfo(p *interfaces.PageBasedInfo) *PageInfo {
	if p == nil {
		return nil
	}

	return &PageInfo{
		TotalCount:  int(p.TotalCount),
		CurrentPage: &p.CurrentPage,
		TotalPages:  &p.TotalPages,
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

func ToPageBasedPaginationParam(pagination PageBasedPagination) *interfaces.PaginationParam {
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

// func intToInt64(i *int) *int64 {
// 	if i == nil {
// 		return nil
// 	}
// 	return lo.ToPtr(int64(*i))
// }

func FromPageInfo(p *PageInfo) *usecasex.PageInfo {
	if p == nil {
		return &usecasex.PageInfo{}
	}
	return &usecasex.PageInfo{
		TotalCount: int64(p.TotalCount),
	}
}
