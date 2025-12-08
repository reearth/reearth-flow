package cms

import (
	"time"
)

type Project struct {
	CreatedAt   time.Time
	UpdatedAt   time.Time
	Description *string
	License     *string
	Readme      *string
	ID          string
	Name        string
	Alias       string
	WorkspaceID string
	Topics      []string
	Visibility  Visibility
	StarCount   int64
}

type Visibility int

const (
	VisibilityPublic Visibility = iota
	VisibilityPrivate
)

type Asset struct {
	CreatedAt               time.Time
	PreviewType             *string
	ArchiveExtractionStatus *string
	ID                      string
	UUID                    string
	ProjectID               string
	Filename                string
	URL                     string
	Size                    uint64
	Public                  bool
}

type Model struct {
	CreatedAt   time.Time
	UpdatedAt   time.Time
	ID          string
	ProjectID   string
	Name        string
	Description string
	Key         string
	PublicAPIEP string
	EditorURL   string
	Schema      Schema
}

type Schema struct {
	SchemaID string
	Fields   []SchemaField
}

type SchemaField struct {
	Description *string
	FieldID     string
	Name        string
	Key         string
	Type        SchemaFieldType
}

type SchemaFieldType int

const (
	SchemaFieldTypeText SchemaFieldType = iota
	SchemaFieldTypeTextArea
	SchemaFieldTypeRichText
	SchemaFieldTypeMarkdownText
	SchemaFieldTypeAsset
	SchemaFieldTypeDate
	SchemaFieldTypeBool
	SchemaFieldTypeSelect
	SchemaFieldTypeTag
	SchemaFieldTypeInteger
	SchemaFieldTypeNumber
	SchemaFieldTypeReference
	SchemaFieldTypeCheckbox
	SchemaFieldTypeURL
	SchemaFieldTypeGroup
	SchemaFieldTypeGeometryObject
	SchemaFieldTypeGeometryEditor
)

type Item struct {
	CreatedAt time.Time
	UpdatedAt time.Time
	Fields    map[string]interface{}
	ID        string
}

type PageInfo struct {
	Page     int32
	PageSize int32
}

type SortInfo struct {
	Key      string
	Reverted bool
}

type ListProjectsInput struct {
	Keyword      *string
	PageInfo     *PageInfo
	SortInfo     *SortInfo
	WorkspaceIDs []string
	PublicOnly   bool
}

type ListProjectsOutput struct {
	PageInfo   *PageInfo
	Projects   []*Project
	TotalCount int64
}

type GetAssetInput struct {
	AssetID string
}

type ListAssetsInput struct {
	PageInfo  *PageInfo
	SortInfo  *SortInfo
	ProjectID string
}

type ListAssetsOutput struct {
	PageInfo   *PageInfo
	Assets     []*Asset
	TotalCount int64
}

type GetModelInput struct {
	ProjectIDOrAlias string
	ModelIDOrAlias   string
}

type ListModelsInput struct {
	PageInfo  *PageInfo
	SortInfo  *SortInfo
	ProjectID string
}

type ListModelsOutput struct {
	PageInfo   *PageInfo
	Models     []*Model
	TotalCount int64
}

type ListItemsInput struct {
	Keyword   *string
	PageInfo  *PageInfo
	SortInfo  *SortInfo
	ModelID   string
	ProjectID string
}

type ListItemsOutput struct {
	PageInfo   *PageInfo
	Items      []Item
	TotalCount int64
}

type ExportInput struct {
	ProjectID string
	ModelID   string
}

type ExportType int

const (
	ExportTypeJSON ExportType = iota
	ExportTypeGeoJSON
)

type ModelExportInput struct {
	ProjectID  string
	ModelID    string
	ExportType ExportType
}

type ExportOutput struct {
	URL string
}
