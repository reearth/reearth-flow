package e2e

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"regexp"
	"sync/atomic"
	"testing"

	"github.com/gavv/httpexpect/v2"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/testutil/factory"
	pkguser "github.com/reearth/reearth-flow/api/pkg/user"
	usermockrepo "github.com/reearth/reearth-flow/api/pkg/user/mockrepo"
	pkgworkspace "github.com/reearth/reearth-flow/api/pkg/workspace"
	workspacemockrepo "github.com/reearth/reearth-flow/api/pkg/workspace/mockrepo"
	"github.com/stretchr/testify/assert"
	"go.uber.org/mock/gomock"
)

type capturedImport struct {
	DocID string
	Body  []byte
}

type importRequest struct {
	Data []int `json:"data"`
}

var _ *httpexpect.Expect

func startMockWS(t *testing.T) (*httptest.Server, *atomic.Pointer[capturedImport]) {
	t.Helper()
	re := regexp.MustCompile(`^/api/document/([^/]+)/import$`)
	cap := &atomic.Pointer[capturedImport]{}
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			w.WriteHeader(http.StatusMethodNotAllowed)
			return
		}
		m := re.FindStringSubmatch(r.URL.Path)
		if len(m) != 2 {
			w.WriteHeader(http.StatusNotFound)
			return
		}
		b := make([]byte, r.ContentLength)
		_, _ = r.Body.Read(b)
		_ = r.Body.Close()
		cap.Store(&capturedImport{DocID: m[1], Body: b})
		w.WriteHeader(http.StatusOK)
	}))
	return srv, cap
}

func TestImportProject_SendsNumberArrayToWebsocket(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	operatorID := pkguser.NewID()
	operator := factory.NewUser(func(b *pkguser.Builder) {
		b.ID(operatorID)
		b.Name("operator")
		b.Email("operator@e2e.com")
	})
	wid := pkgworkspace.NewID()
	w := factory.NewWorkspace(func(b *pkgworkspace.Builder) { b.ID(wid) })

	mockUserRepo := usermockrepo.NewMockUserRepo(ctrl)
	mockWorkspaceRepo := workspacemockrepo.NewMockWorkspaceRepo(ctrl)
	mockUserRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil).AnyTimes()
	mockWorkspaceRepo.EXPECT().FindByID(gomock.Any(), gomock.Any()).Return(w, nil).AnyTimes()
	mock := &TestMocks{UserRepo: mockUserRepo, WorkspaceRepo: mockWorkspaceRepo}

	ws, cap := startMockWS(t)
	defer ws.Close()

	e, _ := StartGQLServer(t, &config.Config{
		Origins:                  []string{"https://example.com"},
		AuthSrv:                  config.AuthSrvConfig{Disabled: true},
		WebsocketThriftServerURL: ws.URL,
	}, true, true, mock)

	projectID := testCreateProject(t, e, operatorID.String(), wid.String())

	mutation := `mutation ImportProject($projectId: ID!, $data: Bytes!) {\n  importProject(projectId: $projectId, data: $data)\n}`
	variables := map[string]any{
		"projectId": projectID,
		"data":      []int{1, 2, 3, 255},
	}
	req := GraphQLRequest{Query: mutation, Variables: variables}

	res := e.POST("/api/graphql").
		WithHeader("Origin", "https://example.com").
		WithHeader("authorization", "Bearer test").
		WithHeader("X-Reearth-Debug-User", operatorID.String()).
		WithHeader("Content-Type", "application/json").
		WithJSON(req).
		Expect().Status(http.StatusOK).JSON()

	res.Object().Value("data").Object().Value("importProject").Boolean().IsTrue()

	ci := cap.Load()
	if ci == nil {
		t.Fatalf("mock websocket did not capture request")
	}
	var got importRequest
	if err := json.Unmarshal(ci.Body, &got); err != nil {
		t.Fatalf("mock body unmarshal failed: %v body=%s", err, string(ci.Body))
	}
	assert.Equal(t, []int{1, 2, 3, 255}, got.Data)
}
