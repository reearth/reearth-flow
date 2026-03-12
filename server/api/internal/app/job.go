package app

import (
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
)

type JobHandler struct{}

func NewJobHandler() *JobHandler {
	return &JobHandler{}
}

// CancelJob godoc
// @Summary      Cancel a running job
// @Description  Cancel a job that was started via an API trigger. Requires the trigger's auth token.
// @Tags         jobs
// @Accept       json
// @Produce      json
// @Param        jobId    path      string             true  "Job ID (runId returned from trigger execution)"
// @Param        request  body      job.CancelRequest  true  "Cancel request with triggerId and optional authToken"
// @Success      200      {object}  job.CancelResponse "Cancellation response with runID, deploymentID, and status"
// @Failure      400      {object}  object             "Invalid request"
// @Failure      401      {object}  object             "Missing or invalid authentication token"
// @Failure      404      {object}  object             "Job or trigger not found"
// @Failure      500      {object}  object             "Internal server error"
// @Router       /api/jobs/{jobId}/cancel [post]
// @Security     BearerAuth
func (h *JobHandler) CancelJob(c echo.Context) error {
	jobID, err := id.JobIDFrom(c.Param("jobId"))
	if err != nil {
		return c.JSON(http.StatusBadRequest, map[string]string{"error": "invalid job ID"})
	}

	var req job.CancelRequest
	if err := c.Bind(&req); err != nil {
		return c.JSON(http.StatusBadRequest, map[string]string{"error": "invalid request body"})
	}

	if err := req.Validate(); err != nil {
		return c.JSON(http.StatusBadRequest, map[string]string{"error": err.Error()})
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

	triggerID, err := id.TriggerIDFrom(req.TriggerID)
	if err != nil {
		return c.JSON(http.StatusBadRequest, map[string]string{"error": "invalid trigger ID"})
	}

	ctx := c.Request().Context()
	usecases := adapter.Usecases(ctx)

	// Validate the auth token against the trigger
	t, err := usecases.Trigger.FindByID(ctx, triggerID)
	if err != nil {
		return c.JSON(http.StatusNotFound, map[string]string{"error": "trigger not found"})
	}

	if t.AuthToken() == nil || *t.AuthToken() != token {
		return c.JSON(http.StatusUnauthorized, map[string]string{"error": "invalid authentication token"})
	}

	// Verify the job belongs to the same deployment as the trigger
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

	resp := job.CancelResponse{
		RunID:        cancelledJob.ID().String(),
		DeploymentID: cancelledJob.Deployment().String(),
		Status:       string(cancelledJob.Status()),
	}

	return c.JSON(http.StatusOK, resp)
}

func SetupJobRoutes(e *echo.Echo) {
	h := NewJobHandler()
	e.POST("/api/jobs/:jobId/cancel", h.CancelJob)
}
