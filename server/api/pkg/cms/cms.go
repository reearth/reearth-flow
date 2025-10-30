package cms

import (
	"time"
)

type Project struct {
	ID          string
	Name        string
	Alias       string
	Description *string
	License     *string
	Readme      *string
	WorkspaceID string
	Visibility  Visibility
	Topics      []string
	StarCount   int32
	CreatedAt   time.Time
	UpdatedAt   time.Time
}

type Visibility int

const (
	VisibilityPublic Visibility = iota
	VisibilityPrivate
)

type Asset struct {
	ID                      string
	UUID                    string
	ProjectID               string
	Filename                string
	Size                    uint64
	PreviewType             *string
	URL                     string
	ArchiveExtractionStatus *string
	Public                  bool
	CreatedAt               time.Time
}

type Model struct {
	ID          string
	ProjectID   string
	Name        string
	Description string
	Key         string
	Schema      Schema
	PublicAPIEP string
	EditorURL   string
	CreatedAt   time.Time
	UpdatedAt   time.Time
}

type Schema struct {
	SchemaID string
	Fields   []SchemaField
}

type SchemaField struct {
	FieldID     string
	Name        string
	Type        SchemaFieldType
	Key         string
	Description *string
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
	ID        string
	Fields    map[string]interface{}
	CreatedAt time.Time
	UpdatedAt time.Time
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
	WorkspaceIDs []string
	Keyword      *string
	PublicOnly   bool
	PageInfo     *PageInfo
	SortInfo     *SortInfo
}

type ListProjectsOutput struct {
	Projects   []*Project
	TotalCount int64
	PageInfo   *PageInfo
}

type GetAssetInput struct {
	AssetID string
}

type ListAssetsInput struct {
	ProjectID string
	PageInfo  *PageInfo
	SortInfo  *SortInfo
}

type ListAssetsOutput struct {
	Assets     []*Asset
	TotalCount int64
	PageInfo   *PageInfo
}

type GetModelInput struct {
	ProjectIDOrAlias string
	ModelIDOrAlias   string
}

type ListModelsInput struct {
	ProjectID string
	PageInfo  *PageInfo
	SortInfo  *SortInfo
}

type ListModelsOutput struct {
	Models     []*Model
	TotalCount int64
	PageInfo   *PageInfo
}

type ListItemsInput struct {
	ModelID   string
	ProjectID string
	Keyword   *string
	PageInfo  *PageInfo
	SortInfo  *SortInfo
}

type ListItemsOutput struct {
	Items      []Item
	TotalCount int64
	PageInfo   *PageInfo
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
