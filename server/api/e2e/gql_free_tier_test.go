package e2e

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"sync/atomic"
	"testing"
	"time"

	"github.com/gavv/httpexpect/v2"
	usermockrepo "github.com/reearth/reearth-accounts/server/pkg/gqlclient/user/mockrepo"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/policy"
	"github.com/reearth/reearth-flow/api/internal/testutil/factory"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"go.uber.org/mock/gomock"
)

// TestFreeTierHourlySchedule drives the real HTTP policy checker against a
// stand-in for the policy-check endpoint. A workspace whose plan does not allow
// private projects is on the free plan, and may not schedule hourly runs.
func TestFreeTierHourlySchedule(t *testing.T) {
	ctrl := gomock.NewController(t)
	operator := factory.NewUser(func(b *accountsuser.Builder) {})
	userRepo := usermockrepo.NewMockRepo(ctrl)
	userRepo.EXPECT().FindMe(gomock.Any()).Return(operator, nil).AnyTimes()

	var paid atomic.Bool
	policyService := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		var req struct {
			CheckType string `json:"check_type"`
		}
		_ = json.NewDecoder(r.Body).Decode(&req)
		allowed := req.CheckType == "general_private_project_creation" && paid.Load()
		_ = json.NewEncoder(w).Encode(map[string]any{"allowed": allowed, "check_type": req.CheckType, "value": 1})
	}))
	defer policyService.Close()

	e, _ := StartGQLServer(t, &config.Config{
		Origins: []string{"https://example.com"},
		AuthSrv: config.AuthSrvConfig{Disabled: true},
	}, true, true, &TestMocks{
		UserRepo:      userRepo,
		PolicyChecker: policy.NewHTTP(policyService.URL, "token", 5*time.Second),
		FreeTier:      gateway.FreeTierLimits{DisallowHourly: true},
	})

	deploymentID := createTestDeployment(t, e)
	do := func(query string, vars map[string]any) *httpexpect.Object {
		return e.POST("/api/graphql").
			WithHeader("authorization", "Bearer test").
			WithHeader("X-Reearth-Debug-User", uId1.String()).
			WithJSON(GraphQLRequest{Query: query, Variables: vars}).
			Expect().Status(http.StatusOK).JSON().Object()
	}
	createTrigger := func(interval string) *httpexpect.Object {
		return do(`mutation($input: CreateTriggerInput!) { createTrigger(input: $input) { id timeInterval } }`,
			map[string]any{"input": map[string]any{
				"workspaceId":  wId1.String(),
				"deploymentId": deploymentID,
				"description":  "schedule",
				"timeDriverInput": map[string]any{
					"interval": interval,
				},
			}})
	}
	updateTrigger := func(id, interval string) *httpexpect.Object {
		return do(`mutation($input: UpdateTriggerInput!) { updateTrigger(input: $input) { id timeInterval } }`,
			map[string]any{"input": map[string]any{
				"triggerId":       id,
				"timeDriverInput": map[string]any{"interval": interval},
			}})
	}

	t.Run("free workspace", func(t *testing.T) {
		paid.Store(false)

		createTrigger("EVERY_HOUR").Path("$.errors[0].message").String().Contains("does not allow hourly schedules")

		daily := createTrigger("EVERY_DAY").Path("$.data.createTrigger").Object()
		daily.Value("timeInterval").IsEqual("EVERY_DAY")

		updateTrigger(daily.Value("id").String().Raw(), "EVERY_HOUR").
			Path("$.errors[0].message").String().Contains("does not allow hourly schedules")
	})

	t.Run("paid workspace", func(t *testing.T) {
		paid.Store(true)
		createTrigger("EVERY_HOUR").Path("$.data.createTrigger.timeInterval").IsEqual("EVERY_HOUR")
	})
}
