package main

import (
	"log"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearthx/cerbos/generator"
)

func main() {
	resources := rbac.DefineResources()

	var defs []generator.ResourceDefinition
	for _, r := range resources {
		defs = append(defs, r)
	}

	if err := generator.GeneratePolicyFiles(defs, "policies"); err != nil {
		log.Fatalf("Failed to generate policy files: %v", err)
	}
}
