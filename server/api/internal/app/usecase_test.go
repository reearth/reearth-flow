package app

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/labstack/echo/v4"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestUsecaseMiddleware_SharesContainerAcrossRequests pins the singleton
// wiring: the middleware must attach the same *interfaces.Container built at
// boot rather than allocating one per request.
func TestUsecaseMiddleware_SharesContainerAcrossRequests(t *testing.T) {
	t.Parallel()

	uc := &interfaces.Container{}

	e := echo.New()
	e.Use(UsecaseMiddleware(uc))

	var observed []*interfaces.Container
	e.GET("/probe", func(c echo.Context) error {
		observed = append(observed, adapter.Usecases(c.Request().Context()))
		return c.NoContent(http.StatusOK)
	})

	for i := 0; i < 2; i++ {
		req := httptest.NewRequest(http.MethodGet, "/probe", nil)
		rec := httptest.NewRecorder()
		e.ServeHTTP(rec, req)
		require.Equal(t, http.StatusOK, rec.Code)
	}

	require.Len(t, observed, 2)
	assert.Same(t, uc, observed[0])
	assert.Same(t, uc, observed[1])
	assert.Same(t, observed[0], observed[1])
}
