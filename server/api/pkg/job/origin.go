package job

// Origin is where outside Flow a job was started from.
type Origin struct {
	CMSItem *CMSItemOrigin
}

// CMSItemOrigin is the CMS item a run was started for; names are a display snapshot.
type CMSItemOrigin struct {
	ProjectID    string
	ModelID      string
	ItemID       string
	ConnectionID string
	ProjectName  string
	ModelName    string
	ItemTitle    string
}
