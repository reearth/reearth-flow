package e2e

import (
	"net/http"
	"testing"

	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/testutil/factory"
	usermockrepo "github.com/reearth/reearth-flow/api/pkg/user/mockrepo"
	"go.uber.org/mock/gomock"
)

func TestMe(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	workspace := factory.NewWorkspace()
	testUserSubject := "auth0|test-user"
	userEntity := factory.NewUser(func(b *accountsuser.Builder) {
		b.MyWorkspaceID(workspace.ID())
		b.Auths([]string{testUserSubject})
	})

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(userEntity, nil)

	mock := &TestMocks{
		UserRepo: mockUserRepo,
	}

	e := StartServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{
			Disabled: true,
		},
		AccountsApiHost: "http://localhost:8080",
	}, true, true, mock)

	requestBody := GraphQLRequest{
		OperationName: "GetMe",
		Query:         "query GetMe { \n me { \n id \n name \n email \n lang \n myWorkspaceId \n } \n}",
		Variables:     map[string]any{},
	}

	e.POST("/api/graphql").
		WithHeader("Origin", "https://example.com").
		WithHeader("authorization", "Bearer test").
		WithHeader("Content-Type", "application/json").
		WithJSON(requestBody).
		Expect().
		Status(http.StatusOK).
		JSON().
		Object().
		Value("data").Object().
		Value("me").Object().
		HasValue("id", userEntity.ID()).
		HasValue("email", userEntity.Email()).
		HasValue("name", userEntity.Name()).
		HasValue("lang", userEntity.Metadata().Lang()).
		HasValue("myWorkspaceId", workspace.ID())
}
