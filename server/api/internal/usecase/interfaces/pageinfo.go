package interfaces

import "github.com/reearth/reearthx/usecasex"

type PageBasedInfo struct {
	*usecasex.PageInfo
	CurrentPage int
	TotalPages  int
}

func NewPageBasedInfo(totalCount int64, currentPage, pageSize int) *PageBasedInfo {
	totalPages := (int(totalCount) + pageSize - 1) / pageSize
	hasNextPage := currentPage < totalPages
	hasPrevPage := currentPage > 1

	return &PageBasedInfo{
		PageInfo: &usecasex.PageInfo{
			TotalCount:      totalCount,
			HasNextPage:     hasNextPage,
			HasPreviousPage: hasPrevPage,
		},
		CurrentPage: currentPage,
		TotalPages:  totalPages,
	}
}

func (p *PageBasedInfo) ToPageInfo() *usecasex.PageInfo {
	if p == nil {
		return nil
	}
	return p.PageInfo
}
