package config

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestReadConfig(t *testing.T) {
	// Set environment variables for testing
	t.Setenv("REEARTH_FLOW_HOST", "http://example.com")
	t.Setenv("REEARTH_FLOW_HOST_WEB", "http://web.example.com")
	t.Setenv("REEARTH_FLOW_DB", "mongodb://testdb")
	t.Setenv("REEARTH_FLOW_AUTH_ISS", "http://auth.example.com")
	t.Setenv("REEARTH_FLOW_AUTH_AUD", "audience1,audience2")

	// Test with debug mode enabled
	config, err := ReadConfig(true)
	assert.NoError(t, err)
	assert.NotNil(t, config)
	assert.Equal(t, true, config.Dev)
	assert.Equal(t, "http://example.com", config.Host)
	assert.Equal(t, "http://web.example.com", config.HostWeb)
	assert.Equal(t, "mongodb://testdb", config.DB)
	assert.Equal(t, "http://auth.example.com", config.AuthIss)
	assert.Equal(t, "audience1,audience2", config.AuthAud)

	// Test with debug mode disabled
	config, err = ReadConfig(false)
	assert.NoError(t, err)
	assert.NotNil(t, config)
	assert.Equal(t, false, config.Dev)
	assert.Equal(t, "http://example.com", config.Host)
	assert.Equal(t, "http://web.example.com", config.HostWeb)
	assert.Equal(t, "mongodb://testdb", config.DB)
	assert.Equal(t, "http://auth.example.com", config.AuthIss)
	assert.Equal(t, "audience1,audience2", config.AuthAud)
}

func Test_AddHTTPScheme(t *testing.T) {
	assert.Equal(t, "http://a", addHTTPScheme("a"))
	assert.Equal(t, "http://a", addHTTPScheme("http://a"))
	assert.Equal(t, "https://a", addHTTPScheme("https://a"))
}
