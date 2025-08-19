package gqlmodel

import (
	"github.com/reearth/reearth-flow/api/pkg/cms"
	"github.com/reearth/reearthx/util"
)

func CMSProjectFrom(p *cms.Project) *CMSProject {
	if p == nil {
		return nil
	}
	return &CMSProject{
		ID:          ID(p.ID),
		Name:        p.Name,
		Alias:       p.Alias,
		Description: p.Description,
		License:     p.License,
		Readme:      p.Readme,
		WorkspaceID: ID(p.WorkspaceID),
		Visibility:  CMSVisibilityFrom(p.Visibility),
		CreatedAt:   p.CreatedAt,
		UpdatedAt:   p.UpdatedAt,
	}
}

func CMSVisibilityFrom(v cms.Visibility) CMSVisibility {
	switch v {
	case cms.VisibilityPublic:
		return CMSVisibilityPublic
	case cms.VisibilityPrivate:
		return CMSVisibilityPrivate
	default:
		return CMSVisibilityPrivate
	}
}

func CMSModelFrom(m *cms.Model) *CMSModel {
	if m == nil {
		return nil
	}
	return &CMSModel{
		ID:          ID(m.ID),
		ProjectID:   ID(m.ProjectID),
		Name:        m.Name,
		Description: m.Description,
		Key:         m.Key,
		Schema:      CMSSchemaFrom(&m.Schema),
		PublicAPIEp: m.PublicAPIEP,
		EditorURL:   m.EditorURL,
		CreatedAt:   m.CreatedAt,
		UpdatedAt:   m.UpdatedAt,
	}
}

func CMSSchemaFrom(s *cms.Schema) *CMSSchema {
	if s == nil {
		return nil
	}
	return &CMSSchema{
		SchemaID: ID(s.SchemaID),
		Fields:   util.Map(s.Fields, CMSSchemaFieldFrom),
	}
}

func CMSSchemaFieldFrom(f cms.SchemaField) *CMSSchemaField {
	return &CMSSchemaField{
		FieldID:     ID(f.FieldID),
		Name:        f.Name,
		Type:        CMSSchemaFieldTypeFrom(f.Type),
		Key:         f.Key,
		Description: f.Description,
	}
}

func CMSSchemaFieldTypeFrom(t cms.SchemaFieldType) CMSSchemaFieldType {
	switch t {
	case cms.SchemaFieldTypeText:
		return CMSSchemaFieldTypeText
	case cms.SchemaFieldTypeTextArea:
		return CMSSchemaFieldTypeTextarea
	case cms.SchemaFieldTypeRichText:
		return CMSSchemaFieldTypeRichtext
	case cms.SchemaFieldTypeMarkdownText:
		return CMSSchemaFieldTypeMarkdowntext
	case cms.SchemaFieldTypeAsset:
		return CMSSchemaFieldTypeAsset
	case cms.SchemaFieldTypeDate:
		return CMSSchemaFieldTypeDate
	case cms.SchemaFieldTypeBool:
		return CMSSchemaFieldTypeBool
	case cms.SchemaFieldTypeSelect:
		return CMSSchemaFieldTypeSelect
	case cms.SchemaFieldTypeTag:
		return CMSSchemaFieldTypeTag
	case cms.SchemaFieldTypeInteger:
		return CMSSchemaFieldTypeInteger
	case cms.SchemaFieldTypeNumber:
		return CMSSchemaFieldTypeNumber
	case cms.SchemaFieldTypeReference:
		return CMSSchemaFieldTypeReference
	case cms.SchemaFieldTypeCheckbox:
		return CMSSchemaFieldTypeCheckbox
	case cms.SchemaFieldTypeURL:
		return CMSSchemaFieldTypeURL
	case cms.SchemaFieldTypeGroup:
		return CMSSchemaFieldTypeGroup
	case cms.SchemaFieldTypeGeometryObject:
		return CMSSchemaFieldTypeGeometryobject
	case cms.SchemaFieldTypeGeometryEditor:
		return CMSSchemaFieldTypeGeometryeditor
	default:
		return CMSSchemaFieldTypeText
	}
}

func CMSAssetFrom(a *cms.Asset) *CMSAsset {
	if a == nil {
		return nil
	}
	return &CMSAsset{
		ID:                      ID(a.ID),
		UUID:                    a.UUID,
		ProjectID:               ID(a.ProjectID),
		Filename:                a.Filename,
		Size:                    int(a.Size),
		PreviewType:             a.PreviewType,
		URL:                     a.URL,
		ArchiveExtractionStatus: a.ArchiveExtractionStatus,
		Public:                  a.Public,
		CreatedAt:               a.CreatedAt,
	}
}

func CMSItemFrom(i *cms.Item) *CMSItem {
	if i == nil {
		return nil
	}

	// Convert fields to JSON
	return &CMSItem{
		ID:        ID(i.ID),
		Fields:    JSON(i.Fields),
		CreatedAt: i.CreatedAt,
		UpdatedAt: i.UpdatedAt,
	}
}
