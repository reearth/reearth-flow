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
	CreatedAt   time.Time
	UpdatedAt   time.Time
}

type Visibility int

const (
	VisibilityPublic Visibility = iota
	VisibilityPrivate
)

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

type ListProjectsInput struct {
	WorkspaceID string
	PublicOnly  bool
}

type ListModelsInput struct {
	ProjectID string
}

type ListItemsInput struct {
	ModelID   string
	ProjectID string
	Page      *int32
	PageSize  *int32
	Keyword   *string
}

type ListItemsOutput struct {
	Items      []Item
	TotalCount int32
}

type ExportInput struct {
	ProjectID string
	ModelID   string
}

type ExportOutput struct {
	URL string
}
