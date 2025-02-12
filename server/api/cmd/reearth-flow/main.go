package main

import "github.com/reearth/reearth-flow/api/internal/app"

var version = ""

func main() {
	app.Start(debug, version)
}
