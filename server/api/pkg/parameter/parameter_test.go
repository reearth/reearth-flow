package parameter

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestParameter_WithoutValue(t *testing.T) {
	t.Parallel()

	updated := time.Date(2026, 1, 2, 3, 4, 5, 0, time.UTC)
	p, _ := New().
		Name("secret").
		Type(TypeText).
		Required(true).
		DefaultValue("hunter2").
		Config(map[string]any{"k": "v"}).
		UpdatedAt(updated).
		Build()

	got := p.WithoutValue()

	assert.Nil(t, got.DefaultValue())
	assert.Equal(t, p.ID(), got.ID())
	assert.Equal(t, p.Name(), got.Name())
	assert.Equal(t, p.Type(), got.Type())
	assert.Equal(t, p.Required(), got.Required())
	assert.Equal(t, p.Config(), got.Config())
	assert.Equal(t, updated, got.UpdatedAt(), "withholding a value is not an update")
	assert.Equal(t, "hunter2", p.DefaultValue(), "the original is not modified")

	var nilParam *Parameter
	assert.Nil(t, nilParam.WithoutValue())
}

func TestParameter_WithValue(t *testing.T) {
	t.Parallel()

	updated := time.Date(2026, 1, 2, 3, 4, 5, 0, time.UTC)
	p, _ := New().
		Name("area").
		Type(TypeText).
		Required(true).
		DefaultValue("tokyo").
		UpdatedAt(updated).
		Build()

	got := p.WithValue("osaka")

	assert.Equal(t, "osaka", got.DefaultValue())
	assert.Equal(t, p.ID(), got.ID())
	assert.Equal(t, p.Name(), got.Name())
	assert.Equal(t, p.Required(), got.Required())
	assert.Equal(t, updated, got.UpdatedAt(), "recording a run's value is not an update")
	assert.Equal(t, "tokyo", p.DefaultValue(), "the original is not modified")

	var nilParam *Parameter
	assert.Nil(t, nilParam.WithValue("x"))
}

func TestParameter_WithProjectID(t *testing.T) {
	t.Parallel()

	updated := time.Date(2026, 1, 2, 3, 4, 5, 0, time.UTC)
	p, _ := New().Name("area").Type(TypeText).DefaultValue("tokyo").UpdatedAt(updated).Build()
	pid := NewProjectID()

	got := p.WithProjectID(pid)

	assert.Equal(t, pid, got.ProjectID())
	assert.Equal(t, p.ID(), got.ID())
	assert.Equal(t, p.Name(), got.Name())
	assert.Equal(t, "tokyo", got.DefaultValue())
	assert.Equal(t, updated, got.UpdatedAt())
	own := p.ProjectID()
	assert.True(t, own.IsNil(), "the original is not modified")

	var nilParam *Parameter
	assert.Nil(t, nilParam.WithProjectID(pid))
}
