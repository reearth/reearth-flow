package app

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/labstack/echo/v4"
	"github.com/stretchr/testify/assert"
)

func TestGetActionDetailsPercentEncodedName(t *testing.T) {
	resetTestData()
	spaced := Action{Name: "CSV Reader", Type: ActionTypeSource, Description: "d"}
	actionsDataMap[""] = ActionsData{Actions: []Action{spaced}}

	e := echo.New()
	e.GET("/actions/:id", getActionDetails)
	req := httptest.NewRequest(http.MethodGet, "/actions/CSV%20Reader", nil)
	rec := httptest.NewRecorder()
	e.ServeHTTP(rec, req)
	assert.Equal(t, http.StatusOK, rec.Code, "percent-encoded spaced action name should resolve; body: %s", rec.Body.String())
}
