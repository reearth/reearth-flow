package cms

import (
	"context"
	"crypto/tls"
	"fmt"
	"strings"
	"sync"
	"time"

	cmspb "github.com/eukarya-inc/reearth-proto/gen/go/cms/v1"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/cms"
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
	connections map[string]*pooledConnection
	maxSize     int
	maxIdleTime time.Duration
	mu          sync.RWMutex
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
	client   cmspb.ReEarthCMSClient
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

	client := cmspb.NewReEarthCMSClient(conn)
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
	resp, err := c.client.GetProject(ctx, &cmspb.ProjectRequest{
		ProjectIdOrAlias: projectIDOrAlias,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to get project: %w", err)
	}

	return convertProtoToProject(resp.Project), nil
}

func (c *grpcClient) ListProjects(ctx context.Context, input cms.ListProjectsInput) (*cms.ListProjectsOutput, error) {
	req := &cmspb.ListProjectsRequest{
		WorkspaceIds: input.WorkspaceIDs,
		Keyword:      input.Keyword,
		PublicOnly:   input.PublicOnly,
	}

	if input.PageInfo != nil {
		req.PageInfo = &cmspb.PageInfo{
			Page:     input.PageInfo.Page,
			PageSize: input.PageInfo.PageSize,
		}
	}

	if input.SortInfo != nil {
		req.SortInfo = &cmspb.SortInfo{
			Key:      input.SortInfo.Key,
			Reverted: input.SortInfo.Reverted,
		}
	}

	resp, err := c.client.ListProjects(ctx, req)
	if err != nil {
		return nil, fmt.Errorf("failed to list projects: %w", err)
	}

	projects := make([]*cms.Project, len(resp.Projects))
	for i, p := range resp.Projects {
		projects[i] = convertProtoToProject(p)
	}

	output := &cms.ListProjectsOutput{
		Projects:   projects,
		TotalCount: resp.TotalCount,
	}

	if resp.PageInfo != nil {
		output.PageInfo = &cms.PageInfo{
			Page:     resp.PageInfo.Page,
			PageSize: resp.PageInfo.PageSize,
		}
	}

	return output, nil
}

func (c *grpcClient) GetAsset(ctx context.Context, input cms.GetAssetInput) (*cms.Asset, error) {
	resp, err := c.client.GetAsset(ctx, &cmspb.AssetRequest{
		AssetId: input.AssetID,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to get asset: %w", err)
	}

	return convertProtoToAsset(resp.Asset), nil
}

func (c *grpcClient) ListAssets(ctx context.Context, input cms.ListAssetsInput) (*cms.ListAssetsOutput, error) {
	req := &cmspb.ListAssetsRequest{
		ProjectId: input.ProjectID,
	}

	if input.PageInfo != nil {
		req.PageInfo = &cmspb.PageInfo{
			Page:     input.PageInfo.Page,
			PageSize: input.PageInfo.PageSize,
		}
	}

	if input.SortInfo != nil {
		req.SortInfo = &cmspb.SortInfo{
			Key:      input.SortInfo.Key,
			Reverted: input.SortInfo.Reverted,
		}
	}

	resp, err := c.client.ListAssets(ctx, req)
	if err != nil {
		return nil, fmt.Errorf("failed to list assets: %w", err)
	}

	assets := make([]*cms.Asset, len(resp.Assets))
	for i, a := range resp.Assets {
		assets[i] = convertProtoToAsset(a)
	}

	output := &cms.ListAssetsOutput{
		Assets:     assets,
		TotalCount: resp.TotalCount,
	}

	if resp.PageInfo != nil {
		output.PageInfo = &cms.PageInfo{
			Page:     resp.PageInfo.Page,
			PageSize: resp.PageInfo.PageSize,
		}
	}

	return output, nil
}

func (c *grpcClient) GetModel(ctx context.Context, input cms.GetModelInput) (*cms.Model, error) {
	resp, err := c.client.GetModel(ctx, &cmspb.ModelRequest{
		ProjectIdOrAlias: input.ProjectIDOrAlias,
		ModelIdOrAlias:   input.ModelIDOrAlias,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to get model: %w", err)
	}

	return convertProtoToModel(resp.Model), nil
}

func (c *grpcClient) ListModels(ctx context.Context, input cms.ListModelsInput) (*cms.ListModelsOutput, error) {
	req := &cmspb.ListModelsRequest{
		ProjectId: input.ProjectID,
	}

	if input.PageInfo != nil {
		req.PageInfo = &cmspb.PageInfo{
			Page:     input.PageInfo.Page,
			PageSize: input.PageInfo.PageSize,
		}
	}

	if input.SortInfo != nil {
		req.SortInfo = &cmspb.SortInfo{
			Key:      input.SortInfo.Key,
			Reverted: input.SortInfo.Reverted,
		}
	}

	resp, err := c.client.ListModels(ctx, req)
	if err != nil {
		return nil, fmt.Errorf("failed to list models: %w", err)
	}

	models := make([]*cms.Model, len(resp.Models))
	for i, m := range resp.Models {
		models[i] = convertProtoToModel(m)
	}

	output := &cms.ListModelsOutput{
		Models:     models,
		TotalCount: resp.TotalCount,
	}

	if resp.PageInfo != nil {
		output.PageInfo = &cms.PageInfo{
			Page:     resp.PageInfo.Page,
			PageSize: resp.PageInfo.PageSize,
		}
	}

	return output, nil
}

func (c *grpcClient) ListItems(ctx context.Context, input cms.ListItemsInput) (*cms.ListItemsOutput, error) {
	req := &cmspb.ListItemsRequest{
		ModelId:   input.ModelID,
		ProjectId: input.ProjectID,
		Keyword:   input.Keyword,
	}

	if input.PageInfo != nil {
		req.PageInfo = &cmspb.PageInfo{
			Page:     input.PageInfo.Page,
			PageSize: input.PageInfo.PageSize,
		}
	}

	if input.SortInfo != nil {
		req.SortInfo = &cmspb.SortInfo{
			Key:      input.SortInfo.Key,
			Reverted: input.SortInfo.Reverted,
		}
	}

	resp, err := c.client.ListItems(ctx, req)
	if err != nil {
		return nil, fmt.Errorf("failed to list items: %w", err)
	}

	items := make([]cms.Item, len(resp.Items))
	for i, item := range resp.Items {
		items[i] = *convertProtoToItem(item)
	}

	output := &cms.ListItemsOutput{
		Items:      items,
		TotalCount: resp.TotalCount,
	}

	if resp.PageInfo != nil {
		output.PageInfo = &cms.PageInfo{
			Page:     resp.PageInfo.Page,
			PageSize: resp.PageInfo.PageSize,
		}
	}

	return output, nil
}

func (c *grpcClient) GetModelExportURL(ctx context.Context, input cms.ModelExportInput) (*cms.ExportOutput, error) {
	var exportType cmspb.ModelExportRequest_Type
	switch input.ExportType {
	case cms.ExportTypeJSON:
		exportType = cmspb.ModelExportRequest_JSON
	case cms.ExportTypeGeoJSON:
		exportType = cmspb.ModelExportRequest_GEOJSON
	default:
		exportType = cmspb.ModelExportRequest_JSON
	}

	resp, err := c.client.GetModelExportURL(ctx, &cmspb.ModelExportRequest{
		ProjectId:  input.ProjectID,
		ModelId:    input.ModelID,
		ExportType: exportType,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to get export URL: %w", err)
	}

	return &cms.ExportOutput{
		URL: resp.Url,
	}, nil
}

func (c *grpcClient) GetModelGeoJSONExportURL(ctx context.Context, input cms.ExportInput) (*cms.ExportOutput, error) {
	resp, err := c.client.GetModelExportURL(ctx, &cmspb.ModelExportRequest{
		ProjectId:  input.ProjectID,
		ModelId:    input.ModelID,
		ExportType: cmspb.ModelExportRequest_GEOJSON,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to get export URL: %w", err)
	}

	return &cms.ExportOutput{
		URL: resp.Url,
	}, nil
}

func convertProtoToProject(p *cmspb.Project) *cms.Project {
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
		Topics:      p.Topics,
		StarCount:   p.StarCount,
		CreatedAt:   p.CreatedAt.AsTime(),
		UpdatedAt:   p.UpdatedAt.AsTime(),
	}
}

func convertProtoToVisibility(v cmspb.Visibility) cms.Visibility {
	switch v {
	case cmspb.Visibility_PUBLIC:
		return cms.VisibilityPublic
	case cmspb.Visibility_PRIVATE:
		return cms.VisibilityPrivate
	default:
		return cms.VisibilityPrivate
	}
}

func convertProtoToAsset(a *cmspb.Asset) *cms.Asset {
	if a == nil {
		return nil
	}
	return &cms.Asset{
		ID:                      a.Id,
		UUID:                    a.Uuid,
		ProjectID:               a.ProjectId,
		Filename:                a.Filename,
		Size:                    a.Size,
		PreviewType:             a.PreviewType,
		URL:                     a.Url,
		ArchiveExtractionStatus: a.ArchiveExtractionStatus,
		Public:                  a.Public,
		CreatedAt:               a.CreatedAt.AsTime(),
	}
}

func convertProtoToModel(m *cmspb.Model) *cms.Model {
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

func convertProtoToSchema(s *cmspb.Schema) cms.Schema {
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

func convertProtoToSchemaField(f *cmspb.SchemaField) cms.SchemaField {
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

func convertProtoToSchemaFieldType(t cmspb.SchemaField_Type) cms.SchemaFieldType {
	return cms.SchemaFieldType(t)
}

func convertProtoToItem(i *cmspb.Item) *cms.Item {
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

	log.Warnf("Unable to unmarshal Any type: %s, returning raw value", a.TypeUrl)
	return a.Value
}
