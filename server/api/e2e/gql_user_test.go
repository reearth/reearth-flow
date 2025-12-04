package e2e

import (
	"encoding/json"
	"fmt"
	"net/http"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	accountsworkspace "github.com/reearth/reearth-accounts/server/pkg/workspace"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/testutil/factory"
	usermockrepo "github.com/reearth/reearth-flow/api/pkg/user/mockrepo"
	"github.com/stretchr/testify/assert"
	"go.uber.org/mock/gomock"
	"golang.org/x/text/language"
)

var (
	uId1 = accountsid.NewUserID()
	wId1 = accountsid.NewWorkspaceID()
)

func TestUpdateMe(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operatorID := accountsuser.NewID()
	uid := accountsuser.NewID()
	wid := accountsworkspace.NewID()
	operator := factory.NewUser(func(b *accountsuser.Builder) {
		b.ID(uid)
		b.Name("updated")
		b.Email("hoge@test.com")
		md := accountsuser.NewMetadata().
			Lang(language.English).
			MustBuild()
		b.Metadata(md)
		b.MyWorkspaceID(wid)
	})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	gomock.InOrder(
		mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil),
		mockUserRepo.EXPECT().UpdateMe(gomock.Any(), gomock.Any()).Return(operator, nil),
	)
	mock := &TestMocks{
		UserRepo: mockUserRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, true, mock)
	query := `mutation UpdateMe { updateMe(input: {name: "updated", email: "hoge@test.com", lang: "ja", password: "Ajsownndww1", passwordConfirmation: "Ajsownndww1"}){ me { id name email lang auths myWorkspaceId } }}`
	request := GraphQLRequest{
		OperationName: "UpdateMe",
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
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object().Value("data").Object().Value("updateMe").Object().Value("me").Object()
	o.Value("name").String().IsEqual("updated")
	o.Value("email").String().IsEqual("hoge@test.com")
	o.Value("lang").String().IsEqual("en")
	o.Value("myWorkspaceId").String().IsEqual(wid.String())
}

func TestRemoveMyAuth(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operatorID := accountsuser.NewID()
	operator := factory.NewUser(func(b *accountsuser.Builder) {
		b.ID(operatorID)
		b.Name("operator")
		b.Email("operator@e2e.com")
	})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	gomock.InOrder(
		mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil),
		mockUserRepo.EXPECT().RemoveMyAuth(gomock.Any(), gomock.Any()).Return(operator, nil),
	)
	mock := &TestMocks{
		UserRepo: mockUserRepo,
	}
	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, true, mock)

	query := `mutation RemoveMyAuth { removeMyAuth(input: {auth: "reearth"}){ me { id name email lang auths } }}`
	request := GraphQLRequest{
		OperationName: "RemoveMyAuth",
		Query:         query,
	}
	jsonData, err := json.Marshal(request)
	if err != nil {
		assert.NoError(t, err)
	}
	e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object()
}

func TestDeleteMe(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operatorID := accountsuser.NewID()
	uid := accountsuser.NewID()
	operator := factory.NewUser(func(b *accountsuser.Builder) {
		b.ID(uid)
		b.Name("updated")
		b.Email("hoge@test.com")
	})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	gomock.InOrder(
		mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil),
		mockUserRepo.EXPECT().DeleteMe(gomock.Any(), gomock.Any()).Return(nil),
	)
	mock := &TestMocks{
		UserRepo: mockUserRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, true, mock)
	query := fmt.Sprintf(`mutation DeleteMe { deleteMe(input: {userId: "%s"}){ userId }}`, uid)
	request := GraphQLRequest{
		OperationName: "DeleteMe",
		Query:         query,
	}
	jsonData, err := json.Marshal(request)
	if err != nil {
		assert.NoError(t, err)
	}
	e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object()
}

func TestSearchUser(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operatorID := accountsuser.NewID()
	uid := accountsuser.NewID()
	operator := factory.NewUser(func(b *accountsuser.Builder) {
		b.ID(operatorID)
		b.Name("operator")
		b.Email("operator@e2e.com")
	})
	u := factory.NewUser(func(b *accountsuser.Builder) {
		b.ID(uid)
		b.Name("e2e")
		b.Email("e2e@e2e.com")
	})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	gomock.InOrder(
		mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil),
		mockUserRepo.EXPECT().UserByNameOrEmail(gomock.Any(), "e2e").Return(u, nil),
		mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil),
		mockUserRepo.EXPECT().UserByNameOrEmail(gomock.Any(), "notfound").Return(nil, nil),
	)
	mock := &TestMocks{
		UserRepo: mockUserRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, true, mock)
	query := fmt.Sprintf(`query SearchUser { searchUser(nameOrEmail: "%s"){ id name email } }`, "e2e")
	request := GraphQLRequest{
		OperationName: "SearchUser",
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
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object().Value("data").Object().Value("searchUser").Object()
	o.Value("id").String().IsEqual(uid.String())
	o.Value("name").String().IsEqual("e2e")
	o.Value("email").String().IsEqual("e2e@e2e.com")

	query = fmt.Sprintf(`query SearchUser { searchUser(nameOrEmail: "%s"){ id name email } }`, "notfound")
	request = GraphQLRequest{
		OperationName: "SearchUser",
		Query:         query,
	}
	jsonData, err = json.Marshal(request)
	if err != nil {
		assert.NoError(t, err)
	}
	e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object().
		Value("data").Object().Value("searchUser").IsNull()
}

func TestNode(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operatorID := accountsuser.NewID()
	operator := factory.NewUser(func(b *accountsuser.Builder) {
		b.ID(operatorID)
		b.Name("operator")
		b.Email("operator@e2e.com")
	})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil)
	mockUserRepo.EXPECT().FindByIDs(gomock.Any(), gomock.Any()).Return(accountsuser.List{operator}, nil)
	mock := &TestMocks{
		UserRepo: mockUserRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, true, mock)
	query := fmt.Sprintf(` { node(id: "%s", type: USER){ id } }`, operatorID.String())
	request := GraphQLRequest{
		Query: query,
	}
	jsonData, err := json.Marshal(request)
	if err != nil {
		assert.NoError(t, err)
	}
	o := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object().Value("data").Object().Value("node").Object()
	o.Value("id").String().IsEqual(operatorID.String())
}

func TestNodes(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operatorID := accountsuser.NewID()
	operator := factory.NewUser(func(b *accountsuser.Builder) {
		b.ID(operatorID)
		b.Name("operator")
		b.Email("operator@e2e.com")
	})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil)
	mockUserRepo.EXPECT().FindByIDs(gomock.Any(), gomock.Any()).Return(accountsuser.List{operator}, nil)
	mock := &TestMocks{
		UserRepo: mockUserRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, true, mock)
	query := fmt.Sprintf(` { nodes(id: "%s", type: USER){ id } }`, operatorID.String())
	request := GraphQLRequest{
		Query: query,
	}
	jsonData, err := json.Marshal(request)
	if err != nil {
		assert.NoError(t, err)
	}
	o := e.POST("/api/graphql").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithBytes(jsonData).Expect().Status(http.StatusOK).JSON().Object().Value("data").Object().Value("nodes")
	o.Array().ContainsAny(map[string]string{"id": operatorID.String()})
}
