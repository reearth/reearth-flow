package app

import (
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
)

func SetupJobRoutes(e *echo.Echo) {
	internal := e.Group("/internal")

	internal.POST("/jobs/:jobId/status", updateJobStatus)
}

func updateJobStatus(c echo.Context) error {
	ctx := c.Request().Context()
	jobIDStr := c.Param("jobId")

	jobID, err := id.JobIDFrom(jobIDStr)
	if err != nil {
		return c.JSON(http.StatusBadRequest, map[string]string{"error": "Invalid job ID"})
	}

	var notification struct {
		JobID  string `json:"jobId"`
		Status string `json:"status"`
	}
	if err := c.Bind(&notification); err != nil {
		return c.JSON(http.StatusBadRequest, map[string]string{"error": "Invalid request body"})
	}

	if notification.JobID != jobIDStr {
		return c.JSON(http.StatusBadRequest, map[string]string{"error": "Job ID mismatch"})
	}

	usecases := adapter.Usecases(ctx)

	status := job.Status(notification.Status)
	if err := usecases.Job.UpdateJobStatusFromEvent(jobID, status); err != nil {
		return c.JSON(http.StatusInternalServerError, map[string]string{"error": "Failed to update job status"})
	}

	return c.JSON(http.StatusOK, map[string]string{"status": "ok"})
}
