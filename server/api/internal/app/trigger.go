package app

import (
	"errors"
	"math"
	"net/http"
	"strconv"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearthx/log"
)

type TriggerHandler struct {
	scheduler *schedulerAuth
}

func NewTriggerHandler(scheduler *schedulerAuth) *TriggerHandler {
	return &TriggerHandler{scheduler: scheduler}
}

// ExecuteScheduledTrigger godoc
// @Summary      Execute scheduled trigger
// @Description  Execute a time-driven trigger (scheduled job)
// @Tags         triggers
// @Produce      json
// @Param        triggerId  path      string  true  "Trigger ID"
// @Success      200        {object}  object  "Execution response with runID, deploymentID, and status; or, for a run that can never succeed, status skipped with a code naming why"
// @Failure      400        {object}  object  "Invalid trigger ID"
// @Failure      401        {object}  object  "Not a run sent by Cloud Scheduler"
// @Failure      409        {object}  object  "The job for this scheduled time is still being started (code scheduled_run_in_progress)"
// @Failure      500        {object}  object  "Internal server error"
// @Router       /api/triggers/{triggerId}/execute-scheduled [post]
// @Security     BearerAuth
func (h *TriggerHandler) ExecuteScheduledTrigger(c echo.Context) error {
	ctx := c.Request().Context()
	if err := h.scheduler.authenticate(ctx, c.Request().Header.Get(echo.HeaderAuthorization)); err != nil {
		log.Warnfc(ctx, "[Trigger] refused execute-scheduled for trigger %s: %v", c.Param("triggerId"), err)
		return c.JSON(http.StatusUnauthorized, map[string]string{"error": errSchedulerUnauthorized.Error()})
	}

	triggerID, err := id.TriggerIDFrom(c.Param("triggerId"))
	if err != nil {
		return c.JSON(http.StatusBadRequest, map[string]string{"error": "invalid trigger ID"})
	}

	triggerUsecase := adapter.Usecases(c.Request().Context()).Trigger

	job, err := triggerUsecase.ExecuteTimeDrivenTrigger(c.Request().Context(), interfaces.ExecuteTimeDrivenTriggerParam{
		TriggerID:   triggerID,
		ScheduledAt: scheduleTime(c.Request()),
	})
	if err != nil {
		return scheduledTriggerError(c, err)
	}

	resp := trigger.ExecutionResponse{
		RunID:        job.ID().String(),
		DeploymentID: job.Deployment().String(),
		Status:       string(job.Status()),
	}

	return c.JSON(http.StatusOK, resp)
}

// ExecuteTrigger godoc
// @Summary      Execute API trigger
// @Description  Execute an API-driven trigger with optional variables and notification URL
// @Tags         triggers
// @Accept       json
// @Produce      json
// @Param        triggerId  path      string  true  "Trigger ID"
// @Param        request    body      object  true  "Execution request with optional authToken, notificationURL, and variables (with)"
// @Success      200        {object}  object  "Execution response with runID, deploymentID, and status"
// @Failure      400        {object}  object  "Invalid request"
// @Failure      401        {object}  object  "Missing authentication token"
// @Failure      409        {object}  object  "The trigger is disabled (code trigger_disabled)"
// @Failure      429        {object}  object  "No more API trigger calls today (code api_trigger_daily_limit), or no more manual deployment runs today (code deployment_runs_daily_limit); see Retry-After"
// @Failure      500        {object}  object  "Internal server error"
// @Router       /api/triggers/{triggerId}/run [post]
// @Security     BearerAuth
func (h *TriggerHandler) ExecuteTrigger(c echo.Context) error {
	triggerID, err := id.TriggerIDFrom(c.Param("triggerId"))
	if err != nil {
		return c.JSON(http.StatusBadRequest, map[string]string{"error": "invalid trigger ID"})
	}

	var req trigger.ExecutionRequest
	if err := c.Bind(&req); err != nil {
		return c.JSON(http.StatusBadRequest, map[string]string{"error": "invalid request body"})
	}

	var token string
	authHeader := c.Request().Header.Get("Authorization")
	if len(authHeader) > 7 && authHeader[:7] == "Bearer " {
		token = authHeader[7:]
	} else if req.AuthToken != "" {
		token = req.AuthToken
	}

	if token == "" {
		return c.JSON(http.StatusUnauthorized, map[string]string{"error": "missing authentication token"})
	}

	if err := req.Validate(); err != nil {
		return c.JSON(http.StatusBadRequest, map[string]string{"error": err.Error()})
	}

	triggerUsecase := adapter.Usecases(c.Request().Context()).Trigger

	job, err := triggerUsecase.ExecuteAPITrigger(c.Request().Context(), interfaces.ExecuteAPITriggerParam{
		AuthenticationToken: token,
		TriggerID:           triggerID,
		NotificationURL: func() *string {
			if req.NotificationURL != "" {
				return &req.NotificationURL
			}
			return nil
		}(),
		Variables: req.With,
	})
	if err != nil {
		return triggerError(c, err)
	}

	resp := trigger.ExecutionResponse{
		RunID:        job.ID().String(),
		DeploymentID: job.Deployment().String(),
		Status:       string(job.Status()),
	}

	return c.JSON(http.StatusOK, resp)
}

// Codes in a refusal's body, so a caller can tell why it was refused.
const (
	codeAPITriggerDailyLimit     = "api_trigger_daily_limit"
	codeDeploymentRunsDailyLimit = "deployment_runs_daily_limit"
	codeTriggerDisabled          = "trigger_disabled"
	// The job for a scheduled time is still being started; a retry finds it
	// once it is submitted.
	codeScheduledRunInProgress = "scheduled_run_in_progress"
)

// triggerError writes err as the response. A run refused by a plan limit is a
// 429, so callers can tell it from a failure, with Retry-After and a code
// naming the limit; a disabled trigger is a 409 with its own code. A scheduled
// run refused by either is skipped before it gets here (see
// scheduledTriggerError); the scheduler retries any other answer outside 2xx
// on its own backoff and ignores Retry-After (see trigger.RetryPolicy).
func triggerError(c echo.Context, err error) error {
	var limit *interfaces.APITriggerLimitError
	var usage *interfaces.UsageLimitError
	switch {
	case errors.As(err, &limit):
		return refuse(c, err, codeAPITriggerDailyLimit, retryAfterSeconds(limit.ResetAt, time.Now()))
	case errors.As(err, &usage) && errors.Is(err, interfaces.ErrDeploymentRunsPerDayLimit):
		// A user running a trigger that is not API-driven runs its deployment
		// by hand.
		return refuse(c, err, codeDeploymentRunsDailyLimit, retryAfterSeconds(usage.ResetAt, time.Now()))
	case errors.Is(err, interfaces.ErrTriggerDisabled):
		// Not a limit to wait out, so there is no Retry-After: the trigger runs
		// again only once it is enabled.
		return c.JSON(http.StatusConflict, map[string]string{"error": err.Error(), "code": codeTriggerDisabled})
	}
	return c.JSON(http.StatusInternalServerError, map[string]string{"error": err.Error()})
}

// scheduleTimeHeader is the scheduled time Cloud Scheduler sends with each
// call, the same on every retry of one run.
const scheduleTimeHeader = "X-CloudScheduler-ScheduleTime"

// scheduleTime is the scheduled time r was sent for, or nil when it carries
// none.
func scheduleTime(r *http.Request) *time.Time {
	at, err := time.Parse(time.RFC3339, r.Header.Get(scheduleTimeHeader))
	if err != nil {
		return nil
	}
	return &at
}

// scheduledTriggerError writes err as the answer to Cloud Scheduler, which
// retries any answer outside 2xx. A run that can never succeed is answered 200
// with status skipped and a code naming why, so it is not retried; any other
// error is answered as triggerError does, and retried.
func scheduledTriggerError(c echo.Context, err error) error {
	if errors.Is(err, interfaces.ErrScheduledRunInProgress) {
		return c.JSON(http.StatusConflict, map[string]string{"error": err.Error(), "code": codeScheduledRunInProgress})
	}
	code, retry := interfaces.ScheduledRunRefusal(err)
	if retry {
		return triggerError(c, err)
	}
	log.Warnfc(c.Request().Context(), "[Trigger] scheduled run of trigger %s skipped (%s): %v", c.Param("triggerId"), code, err)
	return c.JSON(http.StatusOK, map[string]string{"status": "skipped", "code": string(code), "error": err.Error()})
}

func refuse(c echo.Context, err error, code string, retryAfter int) error {
	c.Response().Header().Set(echo.HeaderRetryAfter, strconv.Itoa(retryAfter))
	return c.JSON(http.StatusTooManyRequests, map[string]string{"error": err.Error(), "code": code})
}

// retryAfterSeconds is the whole seconds from now until resetAt, rounded up so
// a caller never retries early, and at least 1.
func retryAfterSeconds(resetAt, now time.Time) int {
	return max(1, int(math.Ceil(resetAt.Sub(now).Seconds())))
}

func SetupTriggerRoutes(e *echo.Echo, scheduler *schedulerAuth) {
	h := NewTriggerHandler(scheduler)
	// Trigger endpoints are public webhooks callable from any origin.
	// ExecuteTrigger enforces its own Bearer token; ExecuteScheduledTrigger
	// accepts only Cloud Scheduler's OIDC token. OPTIONS routes must be registered
	// explicitly so Echo's router matches preflight requests and the CORS
	// middleware runs for them.
	triggerCORS := middleware.CORSWithConfig(middleware.CORSConfig{
		AllowOrigins: []string{"*"},
		AllowMethods: []string{http.MethodPost, http.MethodOptions},
		AllowHeaders: []string{echo.HeaderOrigin, echo.HeaderContentType, echo.HeaderAuthorization},
	})
	g := e.Group("/api/triggers", triggerCORS)
	noop := func(c echo.Context) error { return c.NoContent(http.StatusNoContent) }
	g.OPTIONS("/:triggerId/run", noop)
	g.OPTIONS("/:triggerId/execute-scheduled", noop)
	g.POST("/:triggerId/run", h.ExecuteTrigger)
	g.POST("/:triggerId/execute-scheduled", h.ExecuteScheduledTrigger)
}
