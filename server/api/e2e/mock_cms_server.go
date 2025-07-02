package e2e

import (
	"context"
	"fmt"
	"net"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/cms/proto"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/timestamppb"
)

type MockCMSServer struct {
	proto.UnimplementedReEarthCMSServer
	projects   map[string]*proto.Project
	models     map[string][]*proto.Model
	items      map[string][]*proto.Item
	server     *grpc.Server
	listener   net.Listener
	serviceUrl string
}

func NewMockCMSServer() (*MockCMSServer, error) {
	listener, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		return nil, err
	}

	server := grpc.NewServer()
	mockServer := &MockCMSServer{
		projects:   make(map[string]*proto.Project),
		models:     make(map[string][]*proto.Model),
		items:      make(map[string][]*proto.Item),
		server:     server,
		listener:   listener,
		serviceUrl: listener.Addr().String(),
	}

	mockServer.seedTestData()

	proto.RegisterReEarthCMSServer(server, mockServer)

	go func() {
		if err := server.Serve(listener); err != nil {
			fmt.Printf("mock CMS server error: %v\n", err)
		}
	}()

	return mockServer, nil
}

func (m *MockCMSServer) seedTestData() {
	now := timestamppb.New(time.Now())

	testProject1 := &proto.Project{
		Id:          "project-123",
		Name:        "Test Project 1",
		Alias:       "test-project-1",
		Description: stringPtr("Test project description"),
		WorkspaceId: "workspace-123",
		Visibility:  proto.Visibility_PUBLIC,
		CreatedAt:   now,
		UpdatedAt:   now,
	}

	testProject2 := &proto.Project{
		Id:          "project-456",
		Name:        "Test Project 2",
		Alias:       "test-project-2",
		Description: stringPtr("Private test project"),
		WorkspaceId: "workspace-456",
		Visibility:  proto.Visibility_PRIVATE,
		CreatedAt:   now,
		UpdatedAt:   now,
	}

	m.projects[testProject1.Id] = testProject1
	m.projects[testProject1.Alias] = testProject1
	m.projects[testProject2.Id] = testProject2
	m.projects[testProject2.Alias] = testProject2

	testModel1 := &proto.Model{
		Id:        "model-123",
		ProjectId: "project-123",
		Name:      "Test Model",
		Key:       "test_model",
		CreatedAt: now,
		UpdatedAt: now,
	}
	m.models["project-123"] = append(m.models["project-123"], testModel1)

	testItem1 := &proto.Item{
		Id:        "item-123",
		CreatedAt: now,
		UpdatedAt: now,
	}
	m.items["model-123"] = append(m.items["model-123"], testItem1)
}

func (m *MockCMSServer) Close() {
	if m.server != nil {
		m.server.Stop()
	}
	if m.listener != nil {
		m.listener.Close()
	}
}

func (m *MockCMSServer) GetServiceURL() string {
	return m.serviceUrl
}

func (m *MockCMSServer) GetProject(ctx context.Context, req *proto.ProjectRequest) (*proto.ProjectResponse, error) {
	project, exists := m.projects[req.ProjectIdOrAlias]
	if !exists {
		return nil, status.Error(codes.NotFound, "project not found")
	}

	return &proto.ProjectResponse{
		Project: project,
	}, nil
}

func (m *MockCMSServer) ListProjects(ctx context.Context, req *proto.ListProjectsRequest) (*proto.ListProjectsResponse, error) {
	var projects []*proto.Project
	for _, project := range m.projects {
		if project.WorkspaceId == req.WorkspaceId {
			if !req.PublicOnly || project.Visibility == proto.Visibility_PUBLIC {
				found := false
				for _, p := range projects {
					if p.Id == project.Id {
						found = true
						break
					}
				}
				if !found {
					projects = append(projects, project)
				}
			}
		}
	}

	return &proto.ListProjectsResponse{
		Projects:   projects,
		TotalCount: int32(len(projects)),
	}, nil
}

func (m *MockCMSServer) CreateProject(ctx context.Context, req *proto.CreateProjectRequest) (*proto.ProjectResponse, error) {
	if _, exists := m.projects[req.Alias]; exists {
		return nil, status.Error(codes.AlreadyExists, "alias already exists")
	}

	now := timestamppb.New(time.Now())
	projectID := fmt.Sprintf("project-%d", time.Now().UnixNano())

	project := &proto.Project{
		Id:          projectID,
		Name:        req.Name,
		Alias:       req.Alias,
		Description: req.Description,
		License:     req.License,
		Readme:      req.Readme,
		WorkspaceId: req.WorkspaceId,
		Visibility:  req.Visibility,
		CreatedAt:   now,
		UpdatedAt:   now,
	}

	m.projects[project.Id] = project
	m.projects[project.Alias] = project

	return &proto.ProjectResponse{
		Project: project,
	}, nil
}

func (m *MockCMSServer) UpdateProject(ctx context.Context, req *proto.UpdateProjectRequest) (*proto.ProjectResponse, error) {
	project, exists := m.projects[req.ProjectId]
	if !exists {
		return nil, status.Error(codes.NotFound, "project not found")
	}

	if req.Alias != nil && *req.Alias != project.Alias {
		delete(m.projects, project.Alias)
	}

	if req.Name != nil {
		project.Name = *req.Name
	}
	if req.Description != nil {
		project.Description = req.Description
	}
	if req.License != nil {
		project.License = req.License
	}
	if req.Readme != nil {
		project.Readme = req.Readme
	}
	if req.Alias != nil {
		project.Alias = *req.Alias
		m.projects[project.Alias] = project
	}
	if req.Visibility != nil {
		project.Visibility = *req.Visibility
	}

	project.UpdatedAt = timestamppb.New(time.Now())

	return &proto.ProjectResponse{
		Project: project,
	}, nil
}

func (m *MockCMSServer) DeleteProject(ctx context.Context, req *proto.DeleteProjectRequest) (*proto.DeleteProjectResponse, error) {
	project, exists := m.projects[req.ProjectId]
	if !exists {
		return nil, status.Error(codes.NotFound, "project not found")
	}

	delete(m.projects, project.Id)
	delete(m.projects, project.Alias)

	return &proto.DeleteProjectResponse{
		ProjectId: req.ProjectId,
	}, nil
}

func (m *MockCMSServer) CheckAliasAvailability(ctx context.Context, req *proto.AliasAvailabilityRequest) (*proto.AliasAvailabilityResponse, error) {
	_, exists := m.projects[req.Alias]
	return &proto.AliasAvailabilityResponse{
		Available: !exists,
	}, nil
}

func (m *MockCMSServer) ListModels(ctx context.Context, req *proto.ListModelsRequest) (*proto.ListModelsResponse, error) {
	models, exists := m.models[req.ProjectId]
	if !exists {
		models = []*proto.Model{}
	}

	return &proto.ListModelsResponse{
		Models:     models,
		TotalCount: int32(len(models)),
	}, nil
}

func (m *MockCMSServer) ListItems(ctx context.Context, req *proto.ListItemsRequest) (*proto.ListItemsResponse, error) {
	items, exists := m.items[req.ModelId]
	if !exists {
		items = []*proto.Item{}
	}

	return &proto.ListItemsResponse{
		Items:      items,
		TotalCount: int32(len(items)),
	}, nil
}

func (m *MockCMSServer) GetModelGeoJSONExportURL(ctx context.Context, req *proto.ExportRequest) (*proto.ExportURLResponse, error) {
	return &proto.ExportURLResponse{
		Url: fmt.Sprintf("https://mock-cms.example.com/export/%s/%s.geojson", req.ProjectId, req.ModelId),
	}, nil
}

func stringPtr(s string) *string {
	return &s
}
