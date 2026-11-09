package app

import (
	"context"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/labstack/echo/v4"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/api/idtoken"
)

const (
	testSchedulerSA       = "reearth-flow-scheduler@project.iam.gserviceaccount.com"
	testSchedulerAudience = "https://api.flow.example.com"
)

// testSchedulerAuth accepts the token "good" for claims, as idtoken.Validate
// would for a valid token, and records the audience it was asked to check.
func testSchedulerAuth(claims map[string]any, gotAudience *string) *schedulerAuth {
	a := newSchedulerAuth(testSchedulerSA, testSchedulerAudience)
	a.validate = func(_ context.Context, token, audience string) (*idtoken.Payload, error) {
		if gotAudience != nil {
			*gotAudience = audience
		}
		if token != "good" {
			return nil, errors.New("idtoken: invalid token")
		}
		return &idtoken.Payload{Audience: audience, Claims: claims}, nil
	}
	return a
}

func schedulerClaims(email string, verified bool) map[string]any {
	return map[string]any{"email": email, "email_verified": verified}
}

func TestSchedulerAuth(t *testing.T) {
	ctx := context.Background()

	t.Run("a token for the scheduler service account is accepted", func(t *testing.T) {
		var audience string
		a := testSchedulerAuth(schedulerClaims(testSchedulerSA, true), &audience)

		require.NoError(t, a.authenticate(ctx, "Bearer good"))
		assert.Equal(t, testSchedulerAudience, audience, "the token's audience is checked against Host")
	})

	for name, tc := range map[string]struct {
		auth          *schedulerAuth
		authorization string
	}{
		"no Authorization header":      {testSchedulerAuth(schedulerClaims(testSchedulerSA, true), nil), ""},
		"not a bearer token":           {testSchedulerAuth(schedulerClaims(testSchedulerSA, true), nil), "Basic good"},
		"empty bearer token":           {testSchedulerAuth(schedulerClaims(testSchedulerSA, true), nil), "Bearer "},
		"a token that does not verify": {testSchedulerAuth(schedulerClaims(testSchedulerSA, true), nil), "Bearer forged"},
		"another service account":      {testSchedulerAuth(schedulerClaims("someone-else@project.iam.gserviceaccount.com", true), nil), "Bearer good"},
		"an unverified email":          {testSchedulerAuth(schedulerClaims(testSchedulerSA, false), nil), "Bearer good"},
		"a token with no email":        {testSchedulerAuth(map[string]any{}, nil), "Bearer good"},
		"no service account configured": {
			func() *schedulerAuth {
				a := testSchedulerAuth(schedulerClaims("", true), nil)
				a.serviceAccount = ""
				return a
			}(),
			"Bearer good",
		},
		"no scheduler auth at all": {nil, "Bearer good"},
	} {
		t.Run(name+" is refused", func(t *testing.T) {
			assert.ErrorIs(t, tc.auth.authenticate(ctx, tc.authorization), errSchedulerUnauthorized)
		})
	}
}

func TestExecuteScheduledTrigger_RefusesCallersOtherThanTheScheduler(t *testing.T) {
	e := echo.New()
	SetupTriggerRoutes(e, testSchedulerAuth(schedulerClaims(testSchedulerSA, true), nil))

	for name, authorization := range map[string]string{
		"no token":     "",
		"forged token": "Bearer forged",
	} {
		t.Run(name, func(t *testing.T) {
			req := httptest.NewRequest(http.MethodPost, "/api/triggers/01jxxxxxxxxxxxxxxxxxxxxxxx/execute-scheduled", nil)
			if authorization != "" {
				req.Header.Set(echo.HeaderAuthorization, authorization)
			}
			rec := httptest.NewRecorder()

			e.ServeHTTP(rec, req)

			assert.Equal(t, http.StatusUnauthorized, rec.Code)
		})
	}

	t.Run("the scheduler's token gets past authentication", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodPost, "/api/triggers/not-a-trigger-id/execute-scheduled", nil)
		req.Header.Set(echo.HeaderAuthorization, "Bearer good")
		rec := httptest.NewRecorder()

		e.ServeHTTP(rec, req)

		// The trigger ID is checked only after authentication.
		assert.Equal(t, http.StatusBadRequest, rec.Code)
	})
}
