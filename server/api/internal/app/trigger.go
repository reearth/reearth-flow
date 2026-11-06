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
	"github.com/reearth/reearthx/rerror"
)

type TriggerHandler struct{}

func NewTriggerHandler() *TriggerHandler {
	return &TriggerHandler{}
}

// ExecuteScheduledTrigger godoc
// @Summary      Execute scheduled trigger
// @Description  Execute a time-driven trigger (scheduled job)
// @Tags         triggers
// @Produce      json
// @Param        triggerId  path      string  true  "Trigger ID"
// @Success      200        {object}  object  "Execution response with runID, deploymentID, and status; or, for a run that can never succeed, status skipped with a code naming why"
// @Failure      400        {object}  object  "Invalid trigger ID"
// @Failure      429        {object}  object  "The workspace already has as many jobs running as its plan allows (code concurrent_job_limit, see Retry-After)"
// @Failure      500        {object}  object  "Internal server error"
// @Router       /api/triggers/{triggerId}/execute-scheduled [post]
func (h *TriggerHandler) ExecuteScheduledTrigger(c echo.Context) error {
	triggerID, err := id.TriggerIDFrom(c.Param("triggerId"))
	if err != nil {
		return c.JSON(http.StatusBadRequest, map[string]string{"error": "invalid trigger ID"})
	}

	triggerUsecase := adapter.Usecases(c.Request().Context()).Trigger

	job, err := triggerUsecase.ExecuteTimeDrivenTrigger(c.Request().Context(), interfaces.ExecuteTimeDrivenTriggerParam{
		TriggerID: triggerID,
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
// @Failure      429        {object}  object  "The workspace already has as many jobs running as its plan allows (code concurrent_job_limit), or no more API trigger calls today (code api_trigger_daily_limit); see Retry-After"
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

// Codes in a refusal's body, so a caller can tell the plan's limits apart.
const (
	codeConcurrentJobLimit   = "concurrent_job_limit"
	codeAPITriggerDailyLimit = "api_trigger_daily_limit"
)

// concurrentJobRetryAfter is when a caller refused for running jobs should try
// again. A job's end cannot be known, and a retry still counts as an API
// trigger call, so it is long enough that a caller following it does not use
// up the day's calls on one job.
const concurrentJobRetryAfter = 5 * time.Minute

// triggerError writes err as the response. A refused run is a 429, so callers
// can tell a plan limit from a failure, with Retry-After and a code naming the
// limit. The scheduler retries any answer outside 2xx on its own backoff and
// ignores Retry-After (see gcpscheduler.retryConfigFor).
func triggerError(c echo.Context, err error) error {
	var limit *interfaces.APITriggerLimitError
	switch {
	case errors.As(err, &limit):
		return refuse(c, err, codeAPITriggerDailyLimit, retryAfterSeconds(limit.ResetAt, time.Now()))
	case errors.Is(err, interfaces.ErrJobAlreadyRunning):
		return refuse(c, err, codeConcurrentJobLimit, int(concurrentJobRetryAfter/time.Second))
	}
	return c.JSON(http.StatusInternalServerError, map[string]string{"error": err.Error()})
}

// scheduledTriggerError writes err as the answer to Cloud Scheduler, which
// retries any answer outside 2xx. A run that can never succeed is answered 200
// with status skipped and a code naming why, so it is not retried; any other
// error is answered as triggerError does, and retried.
func scheduledTriggerError(c echo.Context, err error) error {
	code, ok := skipCode(err)
	if !ok {
		return triggerError(c, err)
	}
	log.Warnfc(c.Request().Context(), "[Trigger] scheduled run of trigger %s skipped (%s): %v", c.Param("triggerId"), code, err)
	return c.JSON(http.StatusOK, map[string]string{"status": "skipped", "code": code, "error": err.Error()})
}

// skipCode names why a scheduled run can never succeed, and reports whether it
// can. Only errors known to be permanent are skipped, so an error added later
// is retried until it is listed here. A plan limit that resets with a period,
// such as a number of runs per day, belongs here: retries end long before it
// resets. A limit on runs at the same time does not, as a run ends.
func skipCode(err error) (string, bool) {
	switch {
	case errors.Is(err, interfaces.ErrTriggerNotFound), errors.Is(err, rerror.ErrNotFound):
		return "not_found", true
	case errors.Is(err, interfaces.ErrTriggerDisabled):
		return "trigger_disabled", true
	case errors.Is(err, interfaces.ErrTriggerNotTimeDriven):
		return "trigger_not_time_driven", true
	case errors.Is(err, interfaces.ErrRequiredVariableMissing),
		errors.Is(err, interfaces.ErrInvalidVariableValue),
		errors.Is(err, interfaces.ErrUnknownVariable):
		return "invalid_variables", true
	case errors.Is(err, interfaces.ErrOperationDenied):
		return "operation_denied", true
	}
	return "", false
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

func SetupTriggerRoutes(e *echo.Echo) {
	h := NewTriggerHandler()
	// Trigger endpoints are public webhooks callable from any origin.
	// ExecuteTrigger enforces its own Bearer token; ExecuteScheduledTrigger is
	// intended for the internal scheduler only. OPTIONS routes must be registered
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
