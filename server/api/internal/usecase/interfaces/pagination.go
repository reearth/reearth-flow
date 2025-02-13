package interfaces

// PageBasedPaginationParam represents page-based pagination parameters
type PageBasedPaginationParam struct {
	Page     int
	PageSize int
	OrderBy  *string
	OrderDir *string
}

// PaginationParam represents pagination parameters
type PaginationParam struct {
	Page *PageBasedPaginationParam
}
