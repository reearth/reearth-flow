package app

import (
	"fmt"
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	http1 "github.com/reearth/reearth-flow/api/internal/adapter/http"
)

// Ping godoc
// @Summary      Ping endpoint
// @Description  Health check endpoint that returns "pong"
// @Tags         health
// @Produce      json
// @Success      200  {string}  string  "pong"
// @Router       /api/ping [get]
func Ping() echo.HandlerFunc {
	return func(c echo.Context) error {
		return c.JSON(http.StatusOK, "pong")
	}
}

// Signup godoc
// @Summary      User signup
// @Description  Register a new user account
// @Tags         auth
// @Accept       json
// @Produce      json
// @Param        signup  body      object  true  "Signup information"
// @Success      200     {object}  object  "Signup successful"
// @Failure      400     {object}  object  "Invalid request"
// @Router       /api/signup [post]
// @Security     BearerAuth
func Signup() echo.HandlerFunc {
	return func(c echo.Context) error {
		var inp http1.SignupInput
		if err := c.Bind(&inp); err != nil {
			return &echo.HTTPError{Code: http.StatusBadRequest, Message: fmt.Errorf("failed to parse request body: %w", err)}
		}

		uc := adapter.Usecases(c.Request().Context())
		controller := http1.NewUserController(uc.User)

		output, err := controller.Signup(c.Request().Context(), inp)
		if err != nil {
			return err
		}

		return c.JSON(http.StatusOK, output)
	}
}

// PasswordReset godoc
// @Summary      Password reset
// @Description  Request password reset or set new password
// @Tags         auth
// @Accept       json
// @Produce      json
// @Param        reset  body      object  true  "Password reset information (email only for request, email+token+password for reset)"
// @Success      200    {object}  object  "Success message"
// @Failure      400    {object}  object  "Bad request"
// @Router       /api/password-reset [post]
func PasswordReset() echo.HandlerFunc {
	return func(c echo.Context) error {
		var inp http1.PasswordResetInput
		if err := c.Bind(&inp); err != nil {
			return err
		}

		uc := adapter.Usecases(c.Request().Context())
		controller := http1.NewUserController(uc.User)

		isStartingNewRequest := len(inp.Email) > 0 && len(inp.Token) == 0 && len(inp.Password) == 0
		isSettingNewPassword := len(inp.Email) > 0 && len(inp.Token) > 0 && len(inp.Password) > 0

		if isStartingNewRequest {
			if err := controller.StartPasswordReset(c.Request().Context(), inp); err != nil {
				c.Logger().Error("an attempt to start reset password failed. internal error: %w", err)
			}
			return c.JSON(http.StatusOK, echo.Map{"message": "If that email address is in our database, we will send you an email to reset your password."})
		}

		if isSettingNewPassword {
			if err := controller.PasswordReset(c.Request().Context(), inp); err != nil {
				c.Logger().Error("an attempt to Set password failed. internal error: %w", err)
				return c.JSON(http.StatusBadRequest, echo.Map{"message": "Bad set password request"})
			}
			return c.JSON(http.StatusOK, echo.Map{"message": "Password is updated successfully"})
		}

		return &echo.HTTPError{Code: http.StatusBadRequest, Message: "Bad reset password request"}
	}
}

// StartSignupVerify godoc
// @Summary      Start signup verification
// @Description  Create a verification request for user signup
// @Tags         auth
// @Accept       json
// @Produce      json
// @Param        verify  body  object  true  "Verification request information"
// @Success      200     "Verification created"
// @Failure      400     {object}  object  "Invalid request"
// @Router       /api/signup/verify [post]
// @Security     BearerAuth
func StartSignupVerify() echo.HandlerFunc {
	return func(c echo.Context) error {
		var inp http1.CreateVerificationInput
		if err := c.Bind(&inp); err != nil {
			return &echo.HTTPError{Code: http.StatusBadRequest, Message: fmt.Errorf("failed to parse request body: %w", err)}
		}

		uc := adapter.Usecases(c.Request().Context())
		controller := http1.NewUserController(uc.User)

		if err := controller.CreateVerification(c.Request().Context(), inp); err != nil {
			return err
		}

		return c.NoContent(http.StatusOK)
	}
}

// SignupVerify godoc
// @Summary      Verify signup code
// @Description  Verify user signup with verification code
// @Tags         auth
// @Produce      json
// @Param        code  path      string  true  "Verification code"
// @Success      200   {object}  object  "Verification successful"
// @Failure      400   {object}  object  "Invalid code"
// @Router       /api/signup/verify/{code} [post]
// @Security     BearerAuth
func SignupVerify() echo.HandlerFunc {
	return func(c echo.Context) error {
		code := c.Param("code")
		if len(code) == 0 {
			return echo.ErrBadRequest
		}

		uc := adapter.Usecases(c.Request().Context())
		controller := http1.NewUserController(uc.User)

		output, err := controller.VerifyUser(c.Request().Context(), code)
		if err != nil {
			return err
		}

		return c.JSON(http.StatusOK, output)
	}
}
