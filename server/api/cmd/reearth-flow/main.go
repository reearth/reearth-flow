package main

import "github.com/reearth/reearth-flow/api/internal/app"

var version = ""

// @title           Reearth Flow API
// @version         1.0
// @description     API server for Reearth Flow - workflow automation and data processing platform
// @termsOfService  https://reearth.io/terms/

// @contact.name   Reearth Flow Support
// @contact.url    https://github.com/reearth/reearth-flow
// @contact.email  support@reearth.io

// @license.name  Apache 2.0
// @license.url   http://www.apache.org/licenses/LICENSE-2.0.html

// @host      localhost:8080
// @BasePath  /

// @securityDefinitions.apikey BearerAuth
// @in header
// @name Authorization
// @description Type "Bearer" followed by a space and JWT token.

func main() {
	app.Start(debug, version)
}
