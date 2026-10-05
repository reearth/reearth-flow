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
	assert.Equal(t, "http://web.example.com", config.Host_Web)
	assert.Equal(t, "mongodb://testdb", config.DB)
	assert.Equal(t, "http://auth.example.com", config.Auth_ISS)
	assert.Equal(t, "audience1,audience2", config.Auth_AUD)

	// Test with debug mode disabled
	config, err = ReadConfig(false)
	assert.NoError(t, err)
	assert.NotNil(t, config)
	assert.Equal(t, false, config.Dev)
	assert.Equal(t, "http://example.com", config.Host)
	assert.Equal(t, "http://web.example.com", config.Host_Web)
	assert.Equal(t, "mongodb://testdb", config.DB)
	assert.Equal(t, "http://auth.example.com", config.Auth_ISS)
	assert.Equal(t, "audience1,audience2", config.Auth_AUD)
}

// Feature-view URLs are built from ArtifactBaseURL, so an unset one must point
// at this API's own /artifacts route rather than at a fixed localhost.
func TestReadConfig_ArtifactBaseURL(t *testing.T) {
	t.Run("derived from Host when unset", func(t *testing.T) {
		t.Setenv("REEARTH_FLOW_HOST", "https://api.example.com/")

		config, err := ReadConfig(false)
		assert.NoError(t, err)
		assert.Equal(t, "https://api.example.com/artifacts", config.ArtifactBaseURL)
	})

	t.Run("an explicit value is kept", func(t *testing.T) {
		t.Setenv("REEARTH_FLOW_HOST", "https://api.example.com")
		t.Setenv("REEARTH_FLOW_ARTIFACTBASEURL", "https://cdn.example.com/artifacts")

		config, err := ReadConfig(false)
		assert.NoError(t, err)
		assert.Equal(t, "https://cdn.example.com/artifacts", config.ArtifactBaseURL)
	})
}

func Test_AddHTTPScheme(t *testing.T) {
	assert.Equal(t, "http://a", addHTTPScheme("a"))
	assert.Equal(t, "http://a", addHTTPScheme("http://a"))
	assert.Equal(t, "https://a", addHTTPScheme("https://a"))
}
