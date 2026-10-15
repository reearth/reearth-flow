package interfaces

// PageBasedPaginationParam represents page-based pagination parameters
type PageBasedPaginationParam struct {
	OrderBy  *string
	OrderDir *string
	Page     int
	PageSize int
}

// PaginationParam represents pagination parameters
type PaginationParam struct {
	Page *PageBasedPaginationParam
}

// Pages that callers outside the UI ask for default to DefaultPageSize items
// and hold at most MaxPageSize.
const (
	DefaultPageSize = 20
	MaxPageSize     = 100
)

// NormalizePage returns a 1-based page and a page size within
// [1, MaxPageSize], filling in defaults for values below 1.
func NormalizePage(page, size int) (int, int) {
	if page < 1 {
		page = 1
	}
	if size < 1 {
		size = DefaultPageSize
	}
	return page, min(size, MaxPageSize)
}
