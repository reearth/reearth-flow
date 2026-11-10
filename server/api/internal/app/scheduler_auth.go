package app

import (
	"context"
	"errors"
	"fmt"
	"strings"

	"google.golang.org/api/idtoken"
)

var errSchedulerUnauthorized = errors.New("not a scheduled run")

// schedulerAuth tells a run sent by Cloud Scheduler from any other caller of
// /execute-scheduled. The route is public, so the OIDC token Cloud Scheduler
// signs for serviceAccount is the only proof a request is a scheduled run.
type schedulerAuth struct {
	validate       func(ctx context.Context, token, audience string) (*idtoken.Payload, error)
	serviceAccount string
	audience       string
}

func newSchedulerAuth(serviceAccount, audience string) *schedulerAuth {
	return &schedulerAuth{
		validate:       idtoken.Validate,
		serviceAccount: serviceAccount,
		audience:       audience,
	}
}

// authenticate accepts only a Google-signed, unexpired ID token for audience
// whose email is serviceAccount. With no service account configured it
// refuses everything.
func (a *schedulerAuth) authenticate(ctx context.Context, authorization string) error {
	if a == nil || a.serviceAccount == "" || a.audience == "" {
		return fmt.Errorf("%w: no scheduler service account configured", errSchedulerUnauthorized)
	}
	token, ok := strings.CutPrefix(authorization, "Bearer ")
	if !ok || token == "" {
		return fmt.Errorf("%w: no bearer token", errSchedulerUnauthorized)
	}
	payload, err := a.validate(ctx, token, a.audience)
	if err != nil {
		return fmt.Errorf("%w: %v", errSchedulerUnauthorized, err)
	}
	email, _ := payload.Claims["email"].(string)
	verified, _ := payload.Claims["email_verified"].(bool)
	if !verified || email != a.serviceAccount {
		return fmt.Errorf("%w: token is for %q, not the scheduler service account", errSchedulerUnauthorized, email)
	}
	return nil
}
