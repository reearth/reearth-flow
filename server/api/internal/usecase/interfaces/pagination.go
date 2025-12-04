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
