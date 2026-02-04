package app

import (
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
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
// @Success      200        {object}  object  "Execution response with runID, deploymentID, and status"
// @Failure      400        {object}  object  "Invalid trigger ID"
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
		return c.JSON(http.StatusInternalServerError, map[string]string{"error": err.Error()})
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
		return c.JSON(http.StatusInternalServerError, map[string]string{"error": err.Error()})
	}

	resp := trigger.ExecutionResponse{
		RunID:        job.ID().String(),
		DeploymentID: job.Deployment().String(),
		Status:       string(job.Status()),
	}

	return c.JSON(http.StatusOK, resp)
}

// CancelJob godoc
// @Summary      Cancel a running job
// @Description  Cancel a job that was started via an API trigger. Requires the same Bearer token used to trigger the job.
// @Tags         jobs
// @Accept       json
// @Produce      json
// @Param        jobId    path      string  true  "Job ID (runId returned from trigger execution)"
// @Param        request  body      object  false "Optional request body with triggerId"
// @Success      200      {object}  object  "Cancellation response with runID, deploymentID, and status"
// @Failure      400      {object}  object  "Invalid job ID"
// @Failure      401      {object}  object  "Missing or invalid authentication token"
// @Failure      404      {object}  object  "Job not found"
// @Failure      500      {object}  object  "Internal server error"
// @Router       /api/jobs/{jobId}/cancel [post]
// @Security     BearerAuth
func (h *TriggerHandler) CancelJob(c echo.Context) error {
	jobID, err := id.JobIDFrom(c.Param("jobId"))
	if err != nil {
		return c.JSON(http.StatusBadRequest, map[string]string{"error": "invalid job ID"})
	}

	var token string
	authHeader := c.Request().Header.Get("Authorization")
	if len(authHeader) > 7 && authHeader[:7] == "Bearer " {
		token = authHeader[7:]
	}

	if token == "" {
		return c.JSON(http.StatusUnauthorized, map[string]string{"error": "missing authentication token"})
	}

	var req struct {
		TriggerID string `json:"triggerId"`
	}
	_ = c.Bind(&req)

	if req.TriggerID == "" {
		return c.JSON(http.StatusBadRequest, map[string]string{"error": "triggerId is required in the request body"})
	}

	triggerID, err := id.TriggerIDFrom(req.TriggerID)
	if err != nil {
		return c.JSON(http.StatusBadRequest, map[string]string{"error": "invalid trigger ID"})
	}

	ctx := c.Request().Context()
	usecases := adapter.Usecases(ctx)

	t, err := usecases.Trigger.FindByID(ctx, triggerID)
	if err != nil {
		return c.JSON(http.StatusNotFound, map[string]string{"error": "trigger not found"})
	}

	if t.AuthToken() == nil || *t.AuthToken() != token {
		return c.JSON(http.StatusUnauthorized, map[string]string{"error": "invalid authentication token"})
	}

	j, err := usecases.Job.FindByID(ctx, jobID)
	if err != nil {
		return c.JSON(http.StatusNotFound, map[string]string{"error": "job not found"})
	}

	if j.Deployment() != t.Deployment() {
		return c.JSON(http.StatusUnauthorized, map[string]string{"error": "job does not belong to this trigger's deployment"})
	}

	cancelledJob, err := usecases.Job.Cancel(ctx, jobID)
	if err != nil {
		return c.JSON(http.StatusInternalServerError, map[string]string{"error": err.Error()})
	}

	resp := trigger.ExecutionResponse{
		RunID:        cancelledJob.ID().String(),
		DeploymentID: cancelledJob.Deployment().String(),
		Status:       string(cancelledJob.Status()),
	}

	return c.JSON(http.StatusOK, resp)
}

func SetupTriggerRoutes(e *echo.Echo) {
	h := NewTriggerHandler()
	e.POST("/api/triggers/:triggerId/run", h.ExecuteTrigger)
	e.POST("/api/triggers/:triggerId/execute-scheduled", h.ExecuteScheduledTrigger)
	e.POST("/api/jobs/:jobId/cancel", h.CancelJob)
}
