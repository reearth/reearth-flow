package cms

import (
	"context"
	"crypto/tls"
	"fmt"
	"strings"
	"sync"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/cms"
	"github.com/reearth/reearth-flow/api/pkg/cms/proto"
	"github.com/reearth/reearthx/log"
	"google.golang.org/grpc"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/keepalive"
	protobuf "google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/structpb"
	"google.golang.org/protobuf/types/known/timestamppb"
	"google.golang.org/protobuf/types/known/wrapperspb"
)

var _ gateway.CMS = (*grpcClient)(nil)

type ConnectionPool struct {
	mu          sync.RWMutex
	connections map[string]*pooledConnection
	maxSize     int
	maxIdleTime time.Duration
}

type pooledConnection struct {
	conn     *grpc.ClientConn
	lastUsed time.Time
	refCount int32
}

var (
	globalPool *ConnectionPool
	poolOnce   sync.Once
)

func getGlobalPool() *ConnectionPool {
	poolOnce.Do(func() {
		globalPool = &ConnectionPool{
			connections: make(map[string]*pooledConnection),
			maxSize:     10,
			maxIdleTime: 5 * time.Minute,
		}
		go globalPool.cleanup()
	})
	return globalPool
}

func (p *ConnectionPool) getConnection(endpoint, token string, useTLS bool) (*grpc.ClientConn, error) {
	key := fmt.Sprintf("%s|%s|%t", endpoint, token, useTLS)

	p.mu.RLock()
	if pooled, exists := p.connections[key]; exists {
		if pooled.conn.GetState() == connectivity.Ready || pooled.conn.GetState() == connectivity.Idle {
			pooled.lastUsed = time.Now()
			pooled.refCount++
			p.mu.RUnlock()
			return pooled.conn, nil
		}
	}
	p.mu.RUnlock()

	p.mu.Lock()
	defer p.mu.Unlock()

	if pooled, exists := p.connections[key]; exists {
		if pooled.conn.GetState() == connectivity.Ready || pooled.conn.GetState() == connectivity.Idle {
			pooled.lastUsed = time.Now()
			pooled.refCount++
			return pooled.conn, nil
		}
		pooled.conn.Close()
		delete(p.connections, key)
	}
	var opts []grpc.DialOption

	if useTLS {
		config := &tls.Config{
			ServerName: trimPort(endpoint),
		}
		creds := credentials.NewTLS(config)
		opts = append(opts, grpc.WithTransportCredentials(creds))
	} else {
		opts = append(opts, grpc.WithTransportCredentials(insecure.NewCredentials()))
	}

	if token != "" {
		opts = append(opts, grpc.WithPerRPCCredentials(&tokenAuth{token}))
	}

	opts = append(opts,
		grpc.WithKeepaliveParams(keepalive.ClientParameters{
			Time:                30 * time.Second,
			Timeout:             5 * time.Second,
			PermitWithoutStream: true,
		}),
		grpc.WithDefaultCallOptions(
			grpc.MaxCallRecvMsgSize(4*1024*1024),
			grpc.MaxCallSendMsgSize(4*1024*1024),
		),
	)

	conn, err := grpc.NewClient(endpoint, opts...)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to CMS gRPC server: %w", err)
	}

	p.connections[key] = &pooledConnection{
		conn:     conn,
		lastUsed: time.Now(),
		refCount: 1,
	}

	return conn, nil
}

func (p *ConnectionPool) cleanup() {
	ticker := time.NewTicker(1 * time.Minute)
	defer ticker.Stop()

	for range ticker.C {
		p.mu.Lock()
		now := time.Now()
		for key, pooled := range p.connections {
			if pooled.refCount == 0 && now.Sub(pooled.lastUsed) > p.maxIdleTime {
				log.Debugf("Closing idle gRPC connection: %s", key)
				pooled.conn.Close()
				delete(p.connections, key)
			}
		}
		p.mu.Unlock()
	}
}

func (p *ConnectionPool) releaseConnection(endpoint, token string, useTLS bool) {
	key := fmt.Sprintf("%s|%s|%t", endpoint, token, useTLS)

	p.mu.Lock()
	defer p.mu.Unlock()

	if pooled, exists := p.connections[key]; exists {
		if pooled.refCount > 0 {
			pooled.refCount--
		}
	}
}

type tokenAuth struct {
	token string
}

func (t *tokenAuth) GetRequestMetadata(ctx context.Context, uri ...string) (map[string]string, error) {
	return map[string]string{
		"authorization": fmt.Sprintf("Bearer %s", t.token),
	}, nil
}

func (t *tokenAuth) RequireTransportSecurity() bool {
	return true
}

type grpcClient struct {
	client   proto.ReEarthCMSClient
	conn     *grpc.ClientConn
	endpoint string
	token    string
	useTLS   bool
}

func NewGRPCClient(endpoint, token string, use_tls bool) (gateway.CMS, error) {
	if endpoint == "" {
		return nil, fmt.Errorf("CMS endpoint is required")
	}

	pool := getGlobalPool()
	conn, err := pool.getConnection(endpoint, token, use_tls)
	if err != nil {
		return nil, err
	}

	client := proto.NewReEarthCMSClient(conn)
	return &grpcClient{
		client:   client,
		conn:     conn,
		endpoint: endpoint,
		token:    token,
		useTLS:   use_tls,
	}, nil
}

func (c *grpcClient) Close() error {
	pool := getGlobalPool()
	pool.releaseConnection(c.endpoint, c.token, c.useTLS)
	return nil
}

func trimPort(endpoint string) string {
	return strings.Split(endpoint, ":")[0]
}

func (c *grpcClient) GetProject(ctx context.Context, projectIDOrAlias string) (*cms.Project, error) {
	resp, err := c.client.GetProject(ctx, &proto.ProjectRequest{
		ProjectIdOrAlias: projectIDOrAlias,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to get project: %w", err)
	}

	return convertProtoToProject(resp.Project), nil
}

func (c *grpcClient) ListProjects(ctx context.Context, input cms.ListProjectsInput) ([]*cms.Project, int32, error) {
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

func (c *grpcClient) ListModels(ctx context.Context, input cms.ListModelsInput) ([]*cms.Model, int32, error) {
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

func (c *grpcClient) ListItems(ctx context.Context, input cms.ListItemsInput) (*cms.ListItemsOutput, error) {
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

func (c *grpcClient) GetModelGeoJSONExportURL(ctx context.Context, input cms.ExportInput) (*cms.ExportOutput, error) {
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

	var msg protobuf.Message
	if err := anypb.UnmarshalTo(a, msg, protobuf.UnmarshalOptions{}); err == nil {
		log.Debugf("Successfully unmarshaled Any type: %s", a.TypeUrl)
		return msg
	}

	log.Warnf("Unable to unmarshal Any type: %s, returning raw value", a.TypeUrl)
	return a.Value
}
