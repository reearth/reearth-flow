package cms

import (
	"context"
	"fmt"

	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/cms"
	"github.com/reearth/reearth-flow/api/pkg/cms/proto"
	"github.com/reearth/reearthx/log"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/metadata"
	protobuf "google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/structpb"
	"google.golang.org/protobuf/types/known/timestamppb"
	"google.golang.org/protobuf/types/known/wrapperspb"
)

type grpcClient struct {
	conn     *grpc.ClientConn
	client   proto.ReEarthCMSClient
	endpoint string
	token    string // M2M token for authentication
	userID   string // User ID for metadata
}

// NewGRPCClient creates a new CMS gRPC client
func NewGRPCClient(endpoint, token, userID string) (gateway.CMS, error) {
	if endpoint == "" {
		return nil, fmt.Errorf("CMS endpoint is required")
	}

	// Create gRPC connection without timeout to keep connection alive
	conn, err := grpc.Dial(endpoint,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to CMS gRPC server: %w", err)
	}

	client := proto.NewReEarthCMSClient(conn)

	return &grpcClient{
		conn:     conn,
		client:   client,
		endpoint: endpoint,
		token:    token,
		userID:   userID,
	}, nil
}

// addAuthMetadata adds authentication metadata to the context
func (c *grpcClient) addAuthMetadata(ctx context.Context) context.Context {
	md := metadata.New(map[string]string{
		"authorization": fmt.Sprintf("Bearer %s", c.token),
		"user-id":       c.userID,
	})
	return metadata.NewOutgoingContext(ctx, md)
}

// GetProject retrieves a project by ID or alias
func (c *grpcClient) GetProject(ctx context.Context, projectIDOrAlias string) (*cms.Project, error) {
	ctx = c.addAuthMetadata(ctx)

	resp, err := c.client.GetProject(ctx, &proto.ProjectRequest{
		ProjectIdOrAlias: projectIDOrAlias,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to get project: %w", err)
	}

	return convertProtoToProject(resp.Project), nil
}

// ListProjects lists projects
func (c *grpcClient) ListProjects(ctx context.Context, input cms.ListProjectsInput) ([]*cms.Project, int32, error) {
	ctx = c.addAuthMetadata(ctx)

	resp, err := c.client.ListProjects(ctx, &proto.ListProjectsRequest{
		WorkspaceId: input.WorkspaceID,
		PublicOnly:  input.PublicOnly,
	})
	if err != nil {
		return nil, 0, fmt.Errorf("failed to list projects: %w", err)
	}

	projects := make([]*cms.Project, len(resp.Projects))
	for i, p := range resp.Projects {
		projects[i] = convertProtoToProject(p)
	}

	return projects, resp.TotalCount, nil
}

// ListModels lists models for a project
func (c *grpcClient) ListModels(ctx context.Context, input cms.ListModelsInput) ([]*cms.Model, int32, error) {
	ctx = c.addAuthMetadata(ctx)

	resp, err := c.client.ListModels(ctx, &proto.ListModelsRequest{
		ProjectId: input.ProjectID,
	})
	if err != nil {
		return nil, 0, fmt.Errorf("failed to list models: %w", err)
	}

	models := make([]*cms.Model, len(resp.Models))
	for i, m := range resp.Models {
		models[i] = convertProtoToModel(m)
	}

	return models, resp.TotalCount, nil
}

// ListItems lists items for a model
func (c *grpcClient) ListItems(ctx context.Context, input cms.ListItemsInput) (*cms.ListItemsOutput, error) {
	ctx = c.addAuthMetadata(ctx)

	resp, err := c.client.ListItems(ctx, &proto.ListItemsRequest{
		ModelId:   input.ModelID,
		ProjectId: input.ProjectID,
		Page:      input.Page,
		PageSize:  input.PageSize,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to list items: %w", err)
	}

	items := make([]cms.Item, len(resp.Items))
	for i, item := range resp.Items {
		items[i] = *convertProtoToItem(item)
	}

	return &cms.ListItemsOutput{
		Items:      items,
		TotalCount: resp.TotalCount,
	}, nil
}

// GetModelGeoJSONExportURL gets the GeoJSON export URL for a model
func (c *grpcClient) GetModelGeoJSONExportURL(ctx context.Context, input cms.ExportInput) (*cms.ExportOutput, error) {
	ctx = c.addAuthMetadata(ctx)

	resp, err := c.client.GetModelGeoJSONExportURL(ctx, &proto.ExportRequest{
		ProjectId: input.ProjectID,
		ModelId:   input.ModelID,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to get export URL: %w", err)
	}

	return &cms.ExportOutput{
		URL: resp.Url,
	}, nil
}

// Close closes the gRPC connection
func (c *grpcClient) Close() error {
	if c.conn != nil {
		return c.conn.Close()
	}
	return nil
}

// Converter functions between proto and domain types

func convertProtoToProject(p *proto.Project) *cms.Project {
	if p == nil {
		return nil
	}
	return &cms.Project{
		ID:          p.Id,
		Name:        p.Name,
		Alias:       p.Alias,
		Description: p.Description,
		License:     p.License,
		Readme:      p.Readme,
		WorkspaceID: p.WorkspaceId,
		Visibility:  convertProtoToVisibility(p.Visibility),
		CreatedAt:   p.CreatedAt.AsTime(),
		UpdatedAt:   p.UpdatedAt.AsTime(),
	}
}

func convertProtoToVisibility(v proto.Visibility) cms.Visibility {
	switch v {
	case proto.Visibility_PUBLIC:
		return cms.VisibilityPublic
	case proto.Visibility_PRIVATE:
		return cms.VisibilityPrivate
	default:
		return cms.VisibilityPrivate
	}
}

func convertProtoToModel(m *proto.Model) *cms.Model {
	if m == nil {
		return nil
	}
	return &cms.Model{
		ID:          m.Id,
		ProjectID:   m.ProjectId,
		Name:        m.Name,
		Description: m.Description,
		Key:         m.Key,
		Schema:      convertProtoToSchema(m.Schema),
		PublicAPIEP: m.PublicApiEp,
		EditorURL:   m.EditorUrl,
		CreatedAt:   m.CreatedAt.AsTime(),
		UpdatedAt:   m.UpdatedAt.AsTime(),
	}
}

func convertProtoToSchema(s *proto.Schema) cms.Schema {
	if s == nil {
		return cms.Schema{}
	}
	fields := make([]cms.SchemaField, len(s.Fields))
	for i, f := range s.Fields {
		fields[i] = convertProtoToSchemaField(f)
	}
	return cms.Schema{
		SchemaID: s.SchemaId,
		Fields:   fields,
	}
}

func convertProtoToSchemaField(f *proto.SchemaField) cms.SchemaField {
	if f == nil {
		return cms.SchemaField{}
	}
	return cms.SchemaField{
		FieldID:     f.FieldId,
		Name:        f.Name,
		Type:        convertProtoToSchemaFieldType(f.Type),
		Key:         f.Key,
		Description: f.Description,
	}
}

func convertProtoToSchemaFieldType(t proto.SchemaFieldType) cms.SchemaFieldType {
	// Direct mapping between proto and domain enums
	return cms.SchemaFieldType(t)
}

func convertProtoToItem(i *proto.Item) *cms.Item {
	if i == nil {
		return nil
	}
	fields := make(map[string]interface{})
	for k, v := range i.Fields {
		fields[k] = convertAnyToInterface(v)
	}
	return &cms.Item{
		ID:        i.Id,
		Fields:    fields,
		CreatedAt: i.CreatedAt.AsTime(),
		UpdatedAt: i.UpdatedAt.AsTime(),
	}
}

func convertAnyToInterface(a *anypb.Any) interface{} {
	if a == nil {
		return nil
	}

	// Handle common well-known types
	switch a.TypeUrl {
	case "type.googleapis.com/google.protobuf.StringValue":
		var sv wrapperspb.StringValue
		if err := protobuf.Unmarshal(a.Value, &sv); err == nil {
			return sv.Value
		}

	case "type.googleapis.com/google.protobuf.Int32Value":
		var iv wrapperspb.Int32Value
		if err := protobuf.Unmarshal(a.Value, &iv); err == nil {
			return iv.Value
		}

	case "type.googleapis.com/google.protobuf.Int64Value":
		var iv wrapperspb.Int64Value
		if err := protobuf.Unmarshal(a.Value, &iv); err == nil {
			return iv.Value
		}

	case "type.googleapis.com/google.protobuf.DoubleValue":
		var dv wrapperspb.DoubleValue
		if err := protobuf.Unmarshal(a.Value, &dv); err == nil {
			return dv.Value
		}

	case "type.googleapis.com/google.protobuf.FloatValue":
		var fv wrapperspb.FloatValue
		if err := protobuf.Unmarshal(a.Value, &fv); err == nil {
			return fv.Value
		}

	case "type.googleapis.com/google.protobuf.BoolValue":
		var bv wrapperspb.BoolValue
		if err := protobuf.Unmarshal(a.Value, &bv); err == nil {
			return bv.Value
		}

	case "type.googleapis.com/google.protobuf.Timestamp":
		var ts timestamppb.Timestamp
		if err := protobuf.Unmarshal(a.Value, &ts); err == nil {
			return ts.AsTime()
		}

	case "type.googleapis.com/google.protobuf.Struct":
		var s structpb.Struct
		if err := protobuf.Unmarshal(a.Value, &s); err == nil {
			return s.AsMap()
		}

	case "type.googleapis.com/google.protobuf.Value":
		var v structpb.Value
		if err := protobuf.Unmarshal(a.Value, &v); err == nil {
			return v.AsInterface()
		}

	case "type.googleapis.com/google.protobuf.ListValue":
		var lv structpb.ListValue
		if err := protobuf.Unmarshal(a.Value, &lv); err == nil {
			return lv.AsSlice()
		}
	}

	// Try to unmarshal as a generic message
	var msg protobuf.Message
	if err := anypb.UnmarshalTo(a, msg, protobuf.UnmarshalOptions{}); err == nil {
		log.Debugf("Successfully unmarshaled Any type: %s", a.TypeUrl)
		return msg
	}

	// If all else fails, log warning and return the raw value
	log.Warnf("Unable to unmarshal Any type: %s, returning raw value", a.TypeUrl)
	return a.Value
}
