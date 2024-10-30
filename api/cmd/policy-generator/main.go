package main

import (
	"log"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearthx/cerbos/generator"
)

func main() {
	if err := generator.GeneratePolicies(
		"flow",
		rbac.DefineResources,
		"policies",
	); err != nil {
		log.Fatalf("Failed to generate policies: %v", err)
	}
}
