package e2e

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/testutil/factory"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	pkguser "github.com/reearth/reearth-flow/api/pkg/user"
	usermockrepo "github.com/reearth/reearth-flow/api/pkg/user/mockrepo"
	pkgworkspace "github.com/reearth/reearth-flow/api/pkg/workspace"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/reearth/reearthx/idx"
	"github.com/stretchr/testify/assert"
	"go.uber.org/mock/gomock"
	"golang.org/x/text/language"
)

var (
	uId1 = accountdomain.NewUserID()
	uId2 = accountdomain.NewUserID()
	uId3 = accountdomain.NewUserID()
	wId1 = accountdomain.NewWorkspaceID()
	wId2 = accountdomain.NewWorkspaceID()
	iId1 = accountdomain.NewIntegrationID()
)

func baseSeederUser(ctx context.Context, r *repo.Container) error {
	auth := user.ReearthSub(uId1.String())
	u := user.New().ID(uId1).
		Name("e2e").
		Email("e2e@e2e.com").
		Auths([]user.Auth{*auth}).
		Workspace(wId1).
		MustBuild()
	if err := r.User.Save(ctx, u); err != nil {
		return err
	}
	u2 := user.New().ID(uId2).
		Name("e2e2").
		Workspace(wId2).
		Email("e2e2@e2e.com").
		MustBuild()
	if err := r.User.Save(ctx, u2); err != nil {
		return err
	}
	u3 := user.New().ID(uId3).
		Name("e2e3").
		Workspace(wId2).
		Email("e2e3@e2e.com").
		MustBuild()
	if err := r.User.Save(ctx, u3); err != nil {
		return err
	}
	roleOwner := workspace.Member{
		Role:      workspace.RoleOwner,
		InvitedBy: uId1,
	}
	roleReader := workspace.Member{
		Role:      workspace.RoleReader,
		InvitedBy: uId2,
	}

	w := workspace.New().ID(wId1).
		Name("e2e").
		Members(map[idx.ID[accountdomain.User]]workspace.Member{
			uId1: roleOwner,
		}).
		Integrations(map[idx.ID[accountdomain.Integration]]workspace.Member{
			iId1: roleOwner,
		}).
		MustBuild()
	if err := r.Workspace.Save(ctx, w); err != nil {
		return err
	}

	w2 := workspace.New().ID(wId2).
		Name("e2e2").
		Members(map[idx.ID[accountdomain.User]]workspace.Member{
			uId1: roleOwner,
			uId3: roleReader,
		}).
		Integrations(map[idx.ID[accountdomain.Integration]]workspace.Member{
			iId1: roleOwner,
		}).
		MustBuild()
	if err := r.Workspace.Save(ctx, w2); err != nil {
		return err
	}

	return nil
}

func TestUpdateMe(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operatorID := pkguser.NewID()
	uid := pkguser.NewID()
	wid := pkgworkspace.NewID()
	operator := factory.NewUser(func(b *pkguser.Builder) {
		b.ID(uid)
		b.Name("updated")
		b.Email("hoge@test.com")
		md := pkguser.NewMetadata().
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
	}, true, baseSeederUser, true, mock)
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

	operatorID := pkguser.NewID()
	operator := factory.NewUser(func(b *pkguser.Builder) {
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
	}, true, baseSeederUser, true, mock)

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

	operatorID := pkguser.NewID()
	uid := pkguser.NewID()
	operator := factory.NewUser(func(b *pkguser.Builder) {
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
	}, true, baseSeederUser, true, mock)
	query := fmt.Sprintf(`mutation DeleteMe { deleteMe(input: {userId: "%s"}){ userId }}`, uId1)
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

	operatorID := pkguser.NewID()
	uid := pkguser.NewID()
	operator := factory.NewUser(func(b *pkguser.Builder) {
		b.ID(operatorID)
		b.Name("operator")
		b.Email("operator@e2e.com")
	})
	u := factory.NewUser(func(b *pkguser.Builder) {
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
	}, true, baseSeederUser, true, mock)
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

	operatorID := pkguser.NewID()
	operator := factory.NewUser(func(b *pkguser.Builder) {
		b.ID(operatorID)
		b.Name("operator")
		b.Email("operator@e2e.com")
	})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockUserRepo.EXPECT().FindByIDs(gomock.Any(), gomock.Any()).Return(pkguser.List{operator}, nil)
	mock := &TestMocks{
		UserRepo: mockUserRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser, true, mock)
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

	operatorID := pkguser.NewID()
	operator := factory.NewUser(func(b *pkguser.Builder) {
		b.ID(operatorID)
		b.Name("operator")
		b.Email("operator@e2e.com")
	})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockUserRepo.EXPECT().FindByIDs(gomock.Any(), gomock.Any()).Return(pkguser.List{operator}, nil)
	mock := &TestMocks{
		UserRepo: mockUserRepo,
	}

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
	}, true, baseSeederUser, true, mock)
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
