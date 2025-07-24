package reearthcms

import (
	"context"
	"fmt"
	"log"
	"strings"

	"crypto/tls"

	"github.com/cloudwego/kitex/client"
	"google.golang.org/grpc/metadata"

	"github.com/cloudwego/kitex/transport"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	v1 "github.com/reearth/reearth-flow/api/kitex_gen/reearth/cms/v1"
	"github.com/reearth/reearth-flow/api/kitex_gen/reearth/cms/v1/reearthcms"
	"github.com/reearth/reearth-flow/api/pkg/cms"
)

type Client struct {
	kitexClient reearthcms.Client
	token       string
}

func NewClient(endpoint, token string, use_tls bool) (gateway.CMS, error) {

	var tlsConfig *tls.Config

	if use_tls {
		tlsConfig = &tls.Config{
			ServerName: trim_port(endpoint),
		}
	}

	kitexClient, err := reearthcms.NewClient(
		endpoint,
		client.WithHostPorts(endpoint),
		client.WithTransportProtocol(transport.GRPC),
		client.WithGRPCTLSConfig(tlsConfig),
	)
	if err != nil {
		return nil, fmt.Errorf("failed to create Kitex client: %w", err)
	}

	return &Client{
		kitexClient: kitexClient,
		token:       token,
	}, nil
}

func trim_port(endpoint string) string {
	parts := strings.Split(endpoint, ":")
	if len(parts) > 1 {
		return parts[0]
	}
	return endpoint
}

func (c *Client) addAuthMetadata(ctx context.Context) context.Context {
	return metadata.AppendToOutgoingContext(ctx, "authorization", fmt.Sprintf("Bearer %s", c.token))
}

func (c *Client) GetProject(ctx context.Context, projectIDOrAlias string) (*cms.Project, error) {
	ctx = c.addAuthMetadata(ctx)
	req := &v1.ProjectRequest{
		ProjectIdOrAlias: projectIDOrAlias,
	}

	log.Printf("Making GetProject request...")
	resp, err := c.kitexClient.GetProject(ctx, req)
	if err != nil {
		log.Printf("GetProject error: %v", err)
		return nil, fmt.Errorf("failed to get project: %w", err)
	}

	if resp.Project == nil {
		return nil, fmt.Errorf("project not found")
	}

	return convertProject(resp.Project), nil
}

func (c *Client) ListProjects(ctx context.Context, input cms.ListProjectsInput) ([]*cms.Project, int32, error) {
	ctx = c.addAuthMetadata(ctx)
	req := &v1.ListProjectsRequest{
		WorkspaceId: input.WorkspaceID,
		PublicOnly:  input.PublicOnly,
	}

	resp, err := c.kitexClient.ListProjects(ctx, req)
	if err != nil {
		return nil, 0, fmt.Errorf("failed to list projects: %w", err)
	}

	projects := make([]*cms.Project, len(resp.Projects))
	for i, p := range resp.Projects {
		projects[i] = convertProject(p)
	}

	return projects, resp.TotalCount, nil
}

func (c *Client) ListModels(ctx context.Context, input cms.ListModelsInput) ([]*cms.Model, int32, error) {
	ctx = c.addAuthMetadata(ctx)
	req := &v1.ListModelsRequest{
		ProjectId: input.ProjectID,
	}

	resp, err := c.kitexClient.ListModels(ctx, req)
	if err != nil {
		return nil, 0, fmt.Errorf("failed to list models: %w", err)
	}

	models := make([]*cms.Model, len(resp.Models))
	for i, m := range resp.Models {
		models[i] = convertModel(m)
	}

	return models, resp.TotalCount, nil
}

func (c *Client) ListItems(ctx context.Context, input cms.ListItemsInput) (*cms.ListItemsOutput, error) {
	ctx = c.addAuthMetadata(ctx)
	req := &v1.ListItemsRequest{
		ModelId:   input.ModelID,
		ProjectId: input.ProjectID,
	}

	if input.Page != nil {
		req.Page = input.Page
	}
	if input.PageSize != nil {
		req.PageSize = input.PageSize
	}

	resp, err := c.kitexClient.ListItems(ctx, req)
	if err != nil {
		return nil, fmt.Errorf("failed to list items: %w", err)
	}

	items := make([]cms.Item, len(resp.Items))
	for i, item := range resp.Items {
		convertedItem := convertItem(item)
		if convertedItem != nil {
			items[i] = *convertedItem
		}
	}

	return &cms.ListItemsOutput{
		Items:      items,
		TotalCount: resp.TotalCount,
	}, nil
}

func (c *Client) GetModelGeoJSONExportURL(ctx context.Context, input cms.ExportInput) (*cms.ExportOutput, error) {
	ctx = c.addAuthMetadata(ctx)
	req := &v1.ExportRequest{
		ProjectId: input.ProjectID,
		ModelId:   input.ModelID,
	}

	resp, err := c.kitexClient.GetModelGeoJSONExportURL(ctx, req)
	if err != nil {
		return nil, fmt.Errorf("failed to get export URL: %w", err)
	}

	return &cms.ExportOutput{
		URL: resp.Url,
	}, nil
}

func convertProject(p *v1.Project) *cms.Project {
	if p == nil {
		return nil
	}

	project := &cms.Project{
		ID:          p.Id,
		Name:        p.Name,
		Alias:       p.Alias,
		WorkspaceID: p.WorkspaceId,
		CreatedAt:   p.CreatedAt.AsTime(),
		UpdatedAt:   p.UpdatedAt.AsTime(),
	}

	if p.Description != nil {
		project.Description = p.Description
	}
	if p.License != nil {
		project.License = p.License
	}
	if p.Readme != nil {
		project.Readme = p.Readme
	}

	switch p.Visibility {
	case v1.Visibility_PUBLIC:
		project.Visibility = cms.VisibilityPublic
	case v1.Visibility_PRIVATE:
		project.Visibility = cms.VisibilityPrivate
	}

	return project
}

func convertModel(m *v1.Model) *cms.Model {
	if m == nil {
		return nil
	}

	model := &cms.Model{
		ID:          m.Id,
		ProjectID:   m.ProjectId,
		Name:        m.Name,
		Description: m.Description,
		Key:         m.Key,
		PublicAPIEP: m.PublicApiEp,
		EditorURL:   m.EditorUrl,
		CreatedAt:   m.CreatedAt.AsTime(),
		UpdatedAt:   m.UpdatedAt.AsTime(),
	}

	if m.Schema != nil {
		model.Schema = convertSchema(m.Schema)
	}

	return model
}

func convertSchema(s *v1.Schema) cms.Schema {
	schema := cms.Schema{
		SchemaID: s.SchemaId,
		Fields:   make([]cms.SchemaField, len(s.Fields)),
	}

	for i, f := range s.Fields {
		schema.Fields[i] = convertSchemaField(f)
	}

	return schema
}

func convertSchemaField(f *v1.SchemaField) cms.SchemaField {
	field := cms.SchemaField{
		FieldID: f.FieldId,
		Name:    f.Name,
		Key:     f.Key,
		Type:    convertSchemaFieldType(f.Type),
	}

	if f.Description != nil {
		field.Description = f.Description
	}

	return field
}

func convertSchemaFieldType(t v1.SchemaFieldType) cms.SchemaFieldType {
	switch t {
	case v1.SchemaFieldType_Text:
		return cms.SchemaFieldTypeText
	case v1.SchemaFieldType_TextArea:
		return cms.SchemaFieldTypeTextArea
	case v1.SchemaFieldType_RichText:
		return cms.SchemaFieldTypeRichText
	case v1.SchemaFieldType_MarkdownText:
		return cms.SchemaFieldTypeMarkdownText
	case v1.SchemaFieldType_Asset:
		return cms.SchemaFieldTypeAsset
	case v1.SchemaFieldType_Date:
		return cms.SchemaFieldTypeDate
	case v1.SchemaFieldType_Bool:
		return cms.SchemaFieldTypeBool
	case v1.SchemaFieldType_Select:
		return cms.SchemaFieldTypeSelect
	case v1.SchemaFieldType_Tag:
		return cms.SchemaFieldTypeTag
	case v1.SchemaFieldType_Integer:
		return cms.SchemaFieldTypeInteger
	case v1.SchemaFieldType_Number:
		return cms.SchemaFieldTypeNumber
	case v1.SchemaFieldType_Reference:
		return cms.SchemaFieldTypeReference
	case v1.SchemaFieldType_Checkbox:
		return cms.SchemaFieldTypeCheckbox
	case v1.SchemaFieldType_URL:
		return cms.SchemaFieldTypeURL
	case v1.SchemaFieldType_Group:
		return cms.SchemaFieldTypeGroup
	case v1.SchemaFieldType_GeometryObject:
		return cms.SchemaFieldTypeGeometryObject
	case v1.SchemaFieldType_GeometryEditor:
		return cms.SchemaFieldTypeGeometryEditor
	default:
		return cms.SchemaFieldTypeText
	}
}

func convertItem(item *v1.Item) *cms.Item {
	if item == nil {
		return nil
	}

	fields := make(map[string]interface{})
	for k, v := range item.Fields {
		var val interface{}
		if v != nil {
			val = v
		}
		fields[k] = val
	}

	return &cms.Item{
		ID:        item.Id,
		Fields:    fields,
		CreatedAt: item.CreatedAt.AsTime(),
		UpdatedAt: item.UpdatedAt.AsTime(),
	}
}
