package mongodoc

import (
	"reflect"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
)

func TestNewParameter(t *testing.T) {
	pid := id.NewParameterID()
	projID := id.NewProjectID()
	now := time.Now().UTC()

	param, err := parameter.New().
		ID(pid).
		ProjectID(projID).
		Name("test-param").
		Type(parameter.TypeText).
		Required(true).
		Value("some value").
		Index(5).
		CreatedAt(now).
		UpdatedAt(now).
		Build()
	if err != nil {
		t.Fatalf("unexpected error building parameter: %v", err)
	}

	doc, docID := NewParameter(param)
	if docID != pid.String() {
		t.Errorf("expected docID %s, got %s", pid.String(), docID)
	}

	if doc.ID != pid.String() {
		t.Errorf("expected doc ID %s, got %s", pid.String(), doc.ID)
	}
	if doc.Project != projID.String() {
		t.Errorf("expected project ID %s, got %s", projID.String(), doc.Project)
	}
	if doc.Name != "test-param" {
		t.Errorf("expected name 'test-param', got '%s'", doc.Name)
	}
	if doc.Type != "TEXT" {
		t.Errorf("expected type 'TEXT', got '%s'", doc.Type)
	}
	if doc.Required != true {
		t.Errorf("expected required to be true, got %v", doc.Required)
	}
	if doc.Value != "some value" {
		t.Errorf("expected value 'some value', got '%v'", doc.Value)
	}
	if doc.Index != 5 {
		t.Errorf("expected index 5, got %d", doc.Index)
	}
	if !doc.CreatedAt.Equal(now) {
		t.Errorf("expected createdAt %v, got %v", now, doc.CreatedAt)
	}
	if !doc.UpdatedAt.Equal(now) {
		t.Errorf("expected updatedAt %v, got %v", now, doc.UpdatedAt)
	}
}

func TestParameterDocument_Model(t *testing.T) {
	pid := id.NewParameterID()
	projID := id.NewProjectID()
	now := time.Now().UTC()

	doc := &ParameterDocument{
		CreatedAt: now,
		ID:        pid.String(),
		Index:     3,
		Name:      "my-param",
		Project:   projID.String(),
		Required:  false,
		Type:      "NUMBER",
		UpdatedAt: now,
		Value:     123.45,
	}

	param, err := doc.Model()
	if err != nil {
		t.Fatalf("unexpected error from Model(): %v", err)
	}
	if param == nil {
		t.Fatal("expected parameter, got nil")
	}

	if param.ID() != pid {
		t.Errorf("expected ID %s, got %s", pid.String(), param.ID().String())
	}
	if param.ProjectID() != projID {
		t.Errorf("expected projectID %s, got %s", projID.String(), param.ProjectID().String())
	}
	if param.Name() != "my-param" {
		t.Errorf("expected name 'my-param', got '%s'", param.Name())
	}
	if param.Type() != parameter.TypeNumber {
		t.Errorf("expected type NUMBER, got %s", param.Type())
	}
	if param.Required() != false {
		t.Errorf("expected required false, got %v", param.Required())
	}
	if param.Value() != 123.45 {
		t.Errorf("expected value 123.45, got %v", param.Value())
	}
	if param.Index() != 3 {
		t.Errorf("expected index 3, got %d", param.Index())
	}
	if !param.CreatedAt().Equal(now) {
		t.Errorf("expected createdAt %v, got %v", now, param.CreatedAt())
	}
	if !param.UpdatedAt().Equal(now) {
		t.Errorf("expected updatedAt %v, got %v", now, param.UpdatedAt())
	}
}

func TestNewParameters(t *testing.T) {
	pid1 := id.NewParameterID()
	pid2 := id.NewParameterID()
	projID := id.NewProjectID()
	now := time.Now().UTC()

	param1, err := parameter.New().
		ID(pid1).
		ProjectID(projID).
		Name("param1").
		Type(parameter.TypeText).
		Value("v1").
		Index(0).
		CreatedAt(now).
		UpdatedAt(now).
		Build()
	if err != nil {
		t.Fatalf("unexpected error building param1: %v", err)
	}

	param2, err := parameter.New().
		ID(pid2).
		ProjectID(projID).
		Name("param2").
		Type(parameter.TypeNumber).
		Value(42).
		Index(1).
		CreatedAt(now).
		UpdatedAt(now).
		Build()
	if err != nil {
		t.Fatalf("unexpected error building param2: %v", err)
	}

	params := parameter.ParameterList{param1, param2}
	docs, ids := NewParameters(params)

	if len(docs) != 2 {
		t.Fatalf("expected 2 docs, got %d", len(docs))
	}
	if len(ids) != 2 {
		t.Fatalf("expected 2 ids, got %d", len(ids))
	}

	if ids[0] != pid1.String() || ids[1] != pid2.String() {
		t.Errorf("expected IDs [%s, %s], got [%s, %s]", pid1.String(), pid2.String(), ids[0], ids[1])
	}

	doc1, ok := docs[0].(*ParameterDocument)
	if !ok {
		t.Fatalf("expected doc1 to be *ParameterDocument, got %T", docs[0])
	}
	doc2, ok := docs[1].(*ParameterDocument)
	if !ok {
		t.Fatalf("expected doc2 to be *ParameterDocument, got %T", docs[1])
	}

	if doc1.Name != "param1" || doc1.Value != "v1" || doc1.Type != "TEXT" {
		t.Errorf("doc1 did not match expected values")
	}
	if doc2.Name != "param2" || !reflect.DeepEqual(doc2.Value, 42) || doc2.Type != "NUMBER" {
		t.Errorf("doc2 did not match expected values")
	}
}
