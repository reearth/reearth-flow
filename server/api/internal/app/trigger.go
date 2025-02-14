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

func SetupTriggerRoutes(e *echo.Echo) {
	h := NewTriggerHandler()
	e.POST("/api/triggers/:triggerId/run", h.ExecuteTrigger)
}
