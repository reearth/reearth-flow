package e2e

import (
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"testing"

	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	accountsworkspace "github.com/reearth/reearth-accounts/server/pkg/workspace"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/testutil/factory"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	usermockrepo "github.com/reearth/reearth-flow/api/pkg/user/mockrepo"
	workspacemockrepo "github.com/reearth/reearth-flow/api/pkg/workspace/mockrepo"
	"github.com/stretchr/testify/assert"
	"go.uber.org/mock/gomock"
)

func TestCreateWorkspace(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operatorID := accountsuser.NewID()
	operator := factory.NewUser(func(b *accountsuser.Builder) {
		b.ID(operatorID)
		b.Name("operator")
		b.Email("operator@e2e.com")
	})
	w := factory.NewWorkspace(func(b *accountsworkspace.Builder) {
		b.Name("test")
	})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockWorkspaceRepo := workspacemockrepo.NewMockWorkspaceRepo(ctrl)
	gomock.InOrder(
		mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil),
		mockWorkspaceRepo.EXPECT().Create(gomock.Any(), "test").Return(w, nil),
	)
	mock := &TestMocks{
		UserRepo:      mockUserRepo,
		WorkspaceRepo: mockWorkspaceRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, true, mock)
	query := `mutation CreateWorkspace { createWorkspace(input: {name: "test"}){ workspace{ id name } }}`
	request := GraphQLRequest{
		OperationName: "CreateWorkspace",
		Query:         query,
	}
	jsonData, err := json.Marshal(request)
	if err != nil {
		assert.NoError(t, err)
	}
	o := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object()
	o.Value("data").Object().Value("createWorkspace").Object().Value("workspace").Object().Value("name").String().IsEqual("test")
}

func TestDeleteWorkspace(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operatorID := accountsuser.NewID()
	wid := accountsworkspace.NewID()
	wid2 := accountsworkspace.NewID()
	operator := factory.NewUser(func(b *accountsuser.Builder) {
		b.ID(operatorID)
		b.Name("operator")
		b.Email("operator@e2e.com")
	})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockWorkspaceRepo := workspacemockrepo.NewMockWorkspaceRepo(ctrl)
	gomock.InOrder(
		mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil),
		mockWorkspaceRepo.EXPECT().Delete(gomock.Any(), wid).Return(nil),
		mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil),
		mockWorkspaceRepo.EXPECT().Delete(gomock.Any(), wid2).Return(interfaces.ErrOperationDenied),
	)
	mock := &TestMocks{
		UserRepo:      mockUserRepo,
		WorkspaceRepo: mockWorkspaceRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, true, mock)
	query := fmt.Sprintf(`mutation DeleteWorkspace { deleteWorkspace(input: {workspaceId: "%s"}){ workspaceId }}`, wid)
	request := GraphQLRequest{
		OperationName: "DeleteWorkspace",
		Query:         query,
	}
	jsonData, err := json.Marshal(request)
	assert.Nil(t, err)

	o := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object()
	o.Value("data").Object().Value("deleteWorkspace").Object().Value("workspaceId").String().IsEqual(wid.String())

	query = fmt.Sprintf(`mutation DeleteWorkspace { deleteWorkspace(input: {workspaceId: "%s"}){ workspaceId }}`, wid2)
	request = GraphQLRequest{
		OperationName: "DeleteWorkspace",
		Query:         query,
	}
	jsonData, err = json.Marshal(request)
	assert.Nil(t, err)

	o = e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object()

	o.Value("errors").Array().Value(0).Object().Value("message").IsEqual("input: deleteWorkspace operation denied")
}

func TestUpdateWorkspace(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operatorID := accountsuser.NewID()
	wid := accountsworkspace.NewID()
	wid2 := accountsworkspace.NewID()
	operator := factory.NewUser(func(b *accountsuser.Builder) {
		b.ID(operatorID)
		b.Name("operator")
		b.Email("operator@e2e.com")
	})
	w := factory.NewWorkspace(func(b *accountsworkspace.Builder) {
		b.Name("updated")
	})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockWorkspaceRepo := workspacemockrepo.NewMockWorkspaceRepo(ctrl)
	gomock.InOrder(
		mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil),
		mockWorkspaceRepo.EXPECT().Update(gomock.Any(), wid, "updated").Return(w, nil),
		mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil),
		mockWorkspaceRepo.EXPECT().Update(gomock.Any(), wid2, "updated").Return(nil, errors.New("not found")),
	)
	mock := &TestMocks{
		UserRepo:      mockUserRepo,
		WorkspaceRepo: mockWorkspaceRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, true, mock)

	query := fmt.Sprintf(`mutation UpdateWorkspace { updateWorkspace(input: {workspaceId: "%s",name: "%s"}){ workspace{ id name } }}`, wid, "updated")
	request := GraphQLRequest{
		OperationName: "UpdateWorkspace",
		Query:         query,
	}
	jsonData, err := json.Marshal(request)
	if err != nil {
		assert.Nil(t, err)
	}
	o := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object()
	o.Value("data").Object().Value("updateWorkspace").Object().Value("workspace").Object().Value("name").String().IsEqual("updated")

	query = fmt.Sprintf(`mutation UpdateWorkspace { updateWorkspace(input: {workspaceId: "%s",name: "%s"}){ workspace{ id name } }}`, wid2, "updated")
	request = GraphQLRequest{
		OperationName: "UpdateWorkspace",
		Query:         query,
	}
	jsonData, err = json.Marshal(request)
	if err != nil {
		assert.Nil(t, err)
	}
	o = e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object()
	o.Value("errors").Array().Value(0).Object().Value("message").IsEqual("input: updateWorkspace not found")
}

func TestAddMemberToWorkspace(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operatorID := accountsuser.NewID()
	wid := accountsworkspace.NewID()
	uid := accountsuser.NewID()
	operator := factory.NewUser(func(b *accountsuser.Builder) {
		b.ID(operatorID)
		b.Name("operator")
		b.Email("operator@e2e.com")
	})
	w := factory.NewWorkspace(func(b *accountsworkspace.Builder) {})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockWorkspaceRepo := workspacemockrepo.NewMockWorkspaceRepo(ctrl)
	gomock.InOrder(
		mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil),
		mockWorkspaceRepo.EXPECT().AddUserMember(gomock.Any(), gomock.Any(), gomock.Any()).Return(w, nil),
		mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil),
		mockWorkspaceRepo.EXPECT().AddUserMember(gomock.Any(), gomock.Any(), gomock.Any()).Return(nil, errors.New("user already joined")),
	)
	mock := &TestMocks{
		UserRepo:      mockUserRepo,
		WorkspaceRepo: mockWorkspaceRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, true, mock)

	query := fmt.Sprintf(`mutation AddMemberToWorkspace { addMemberToWorkspace(input: {workspaceId: "%s", userId: "%s", role: READER}){ workspace{ id } }}`, wid, uid)
	request := GraphQLRequest{
		OperationName: "AddMemberToWorkspace",
		Query:         query,
	}
	jsonData, err := json.Marshal(request)
	if err != nil {
		assert.Nil(t, err)
	}
	e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK)

	query = fmt.Sprintf(`mutation AddMemberToWorkspace { addMemberToWorkspace(input: {workspaceId: "%s", userId: "%s", role: READER}){ workspace{ id } }}`, wid, uid)
	request = GraphQLRequest{
		OperationName: "AddMemberToWorkspace",
		Query:         query,
	}
	jsonData, err = json.Marshal(request)
	if err != nil {
		assert.Nil(t, err)
	}
	e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object().
		Value("errors").Array().Value(0).Object().Value("message").IsEqual("input: addMemberToWorkspace user already joined")
}

func TestRemoveMemberFromWorkspace(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operatorID := accountsuser.NewID()
	wid := accountsworkspace.NewID()
	uid := accountsuser.NewID()
	operator := factory.NewUser(func(b *accountsuser.Builder) {
		b.ID(operatorID)
		b.Name("operator")
		b.Email("operator@e2e.com")
	})
	w := factory.NewWorkspace(func(b *accountsworkspace.Builder) {})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockWorkspaceRepo := workspacemockrepo.NewMockWorkspaceRepo(ctrl)
	gomock.InOrder(
		mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil),
		mockWorkspaceRepo.EXPECT().RemoveUserMember(gomock.Any(), gomock.Any(), gomock.Any()).Return(w, nil),
		mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil),
		mockWorkspaceRepo.EXPECT().RemoveUserMember(gomock.Any(), gomock.Any(), gomock.Any()).Return(nil, errors.New("target user does not exist in the workspace")),
	)
	mock := &TestMocks{
		UserRepo:      mockUserRepo,
		WorkspaceRepo: mockWorkspaceRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, true, mock)

	query := fmt.Sprintf(`mutation RemoveMemberFromWorkspace { removeMemberFromWorkspace(input: {workspaceId: "%s", userId: "%s"}){ workspace{ id } }}`, wid, uid)
	request := GraphQLRequest{
		OperationName: "RemoveMemberFromWorkspace",
		Query:         query,
	}
	jsonData, err := json.Marshal(request)
	if err != nil {
		assert.Nil(t, err)
	}
	e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK)

	o := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object()
	o.Value("errors").Array().Value(0).Object().Value("message").IsEqual("input: removeMemberFromWorkspace target user does not exist in the workspace")
}

func TestUpdateMemberOfWorkspace(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operatorID := accountsuser.NewID()
	wid := accountsworkspace.NewID()
	wid2 := accountsworkspace.NewID()
	uid := accountsuser.NewID()
	uid2 := accountsuser.NewID()
	operator := factory.NewUser(func(b *accountsuser.Builder) {
		b.ID(operatorID)
		b.Name("operator")
		b.Email("operator@e2e.com")
	})
	w := factory.NewWorkspace(func(b *accountsworkspace.Builder) {})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockWorkspaceRepo := workspacemockrepo.NewMockWorkspaceRepo(ctrl)
	gomock.InOrder(
		mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil),
		mockWorkspaceRepo.EXPECT().UpdateUserMember(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(w, nil),
		mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil),
		mockWorkspaceRepo.EXPECT().UpdateUserMember(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil, errors.New("operation denied")),
	)
	mock := &TestMocks{
		UserRepo:      mockUserRepo,
		WorkspaceRepo: mockWorkspaceRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, true, mock)

	query := fmt.Sprintf(`mutation UpdateMemberOfWorkspace { updateMemberOfWorkspace(input: {workspaceId: "%s", userId: "%s", role: WRITER}){ workspace{ id } }}`, wid, uid)
	request := GraphQLRequest{
		OperationName: "UpdateMemberOfWorkspace",
		Query:         query,
	}
	jsonData, err := json.Marshal(request)
	if err != nil {
		assert.Nil(t, err)
	}
	e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK)

	query = fmt.Sprintf(`mutation UpdateMemberOfWorkspace { updateMemberOfWorkspace(input: {workspaceId: "%s", userId: "%s", role: WRITER}){ workspace{ id } }}`, wid2, uid2)
	request = GraphQLRequest{
		OperationName: "UpdateMemberOfWorkspace",
		Query:         query,
	}
	jsonData, err = json.Marshal(request)
	if err != nil {
		assert.Nil(t, err)
	}
	o := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object()
	o.Value("errors").Array().Value(0).Object().Value("message").IsEqual("input: updateMemberOfWorkspace operation denied")
}
