package app

import (
	"fmt"
	"net/http"

	"github.com/labstack/echo/v4"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	http1 "github.com/reearth/reearth-flow/api/internal/adapter/http"
)

func Ping() echo.HandlerFunc {
	return func(c echo.Context) error {
		return c.JSON(http.StatusOK, "pong")
	}
}

func Signup() echo.HandlerFunc {
	return func(c echo.Context) error {
		var inp http1.SignupInput
		if err := c.Bind(&inp); err != nil {
			return &echo.HTTPError{Code: http.StatusBadRequest, Message: fmt.Errorf("failed to parse request body: %w", err)}
		}

		uc := adapter.Usecases(c.Request().Context())
		controller := http1.NewUserController(uc.ReearthxUser)

		output, err := controller.Signup(c.Request().Context(), inp)
		if err != nil {
			return err
		}

		return c.JSON(http.StatusOK, output)
	}
}

func PasswordReset() echo.HandlerFunc {
	return func(c echo.Context) error {
		var inp http1.PasswordResetInput
		if err := c.Bind(&inp); err != nil {
			return err
		}

		uc := adapter.Usecases(c.Request().Context())
		controller := http1.NewUserController(uc.ReearthxUser)

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

func StartSignupVerify() echo.HandlerFunc {
	return func(c echo.Context) error {
		var inp http1.CreateVerificationInput
		if err := c.Bind(&inp); err != nil {
			return &echo.HTTPError{Code: http.StatusBadRequest, Message: fmt.Errorf("failed to parse request body: %w", err)}
		}

		uc := adapter.Usecases(c.Request().Context())
		controller := http1.NewUserController(uc.ReearthxUser)

		if err := controller.CreateVerification(c.Request().Context(), inp); err != nil {
			return err
		}

		return c.NoContent(http.StatusOK)
	}
}

func SignupVerify() echo.HandlerFunc {
	return func(c echo.Context) error {
		code := c.Param("code")
		if len(code) == 0 {
			return echo.ErrBadRequest
		}

		uc := adapter.Usecases(c.Request().Context())
		controller := http1.NewUserController(uc.ReearthxUser)

		output, err := controller.VerifyUser(c.Request().Context(), code)
		if err != nil {
			return err
		}

		return c.JSON(http.StatusOK, output)
	}
}
