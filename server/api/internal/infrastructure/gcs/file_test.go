package gcs

import (
	"net/url"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestGetGCSObjectURL(t *testing.T) {
	e, _ := url.Parse("https://hoge.com/assets/xxx.yyy")
	b, _ := url.Parse("https://hoge.com/assets")
	assert.Equal(t, e, getGCSObjectURL(b, "xxx.yyy"))
}

func TestGetGCSObjectNameFromURL(t *testing.T) {
	u, _ := url.Parse("https://hoge.com/assets/xxx.yyy")
	b, _ := url.Parse("https://hoge.com")
	b2, _ := url.Parse("https://hoge2.com")
	assert.Equal(t, "assets/xxx.yyy", getGCSObjectNameFromURL(b, u, gcsAssetBasePath))
	assert.Equal(t, "", getGCSObjectNameFromURL(b2, u, gcsAssetBasePath))
	assert.Equal(t, "", getGCSObjectNameFromURL(nil, u, gcsAssetBasePath))
	assert.Equal(t, "", getGCSObjectNameFromURL(b, nil, gcsAssetBasePath))
}

func TestGetGCSObjectNameFromURL_WithUnderscores(t *testing.T) {
	tests := []struct {
		name        string
		baseURL     string
		objectURL   string
		gcsBasePath string
		expected    string
	}{
		{
			name:        "simple underscore",
			baseURL:     "https://hoge.com",
			objectURL:   "https://hoge.com/assets/my_file.txt",
			gcsBasePath: "assets",
			expected:    "assets/my_file.txt",
		},
		{
			name:        "multiple underscores",
			baseURL:     "https://hoge.com",
			objectURL:   "https://hoge.com/assets/my_long_file_name.txt",
			gcsBasePath: "assets",
			expected:    "assets/my_long_file_name.txt",
		},
		{
			name:        "different base URL",
			baseURL:     "https://hoge2.com",
			objectURL:   "https://hoge.com/assets/my_file.txt",
			gcsBasePath: "assets",
			expected:    "", // should fail due to different host
		},
		{
			name:        "underscore in path",
			baseURL:     "https://hoge.com",
			objectURL:   "https://hoge.com/assets/folder_name/my_file.txt",
			gcsBasePath: "assets",
			expected:    "assets/folder_name/my_file.txt",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			base, _ := url.Parse(tt.baseURL)
			objectURL, _ := url.Parse(tt.objectURL)
			result := getGCSObjectNameFromURL(base, objectURL, tt.gcsBasePath)
			assert.Equal(t, tt.expected, result)
		})
	}
}

func TestGetGCSObjectURL_WithUnderscores(t *testing.T) {
	tests := []struct {
		name       string
		baseURL    string
		objectName string
		expected   string
	}{
		{
			name:       "simple underscore",
			baseURL:    "https://hoge.com/assets",
			objectName: "my_file.txt",
			expected:   "https://hoge.com/assets/my_file.txt",
		},
		{
			name:       "multiple underscores",
			baseURL:    "https://hoge.com/assets",
			objectName: "my_long_file_name.txt",
			expected:   "https://hoge.com/assets/my_long_file_name.txt",
		},
		{
			name:       "underscore with path",
			baseURL:    "https://hoge.com/assets",
			objectName: "folder/my_file.txt",
			expected:   "https://hoge.com/assets/folder/my_file.txt",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			base, _ := url.Parse(tt.baseURL)
			expected, _ := url.Parse(tt.expected)
			result := getGCSObjectURL(base, tt.objectName)
			assert.Equal(t, expected, result)
		})
	}
}

func TestGetJobLogURL(t *testing.T) {
	baseURL, _ := url.Parse("https://storage.googleapis.com/mybucket")
	repo := &fileRepo{
		bucketName: "mybucket",
		base:       baseURL,
	}

	logURL := repo.GetJobLogURL("job123")
	expected := "https://storage.googleapis.com/mybucket/artifacts/job123/action-log/all.log"
	assert.Equal(t, expected, logURL)
}

func TestGetJobWorkerLogURL(t *testing.T) {
	baseURL, _ := url.Parse("https://storage.googleapis.com/mybucket")
	repo := &fileRepo{
		bucketName: "mybucket",
		base:       baseURL,
	}

	workerLogURL := repo.GetJobWorkerLogURL("job123")
	expected := "https://storage.googleapis.com/mybucket/artifacts/job123/worker/worker.log"
	assert.Equal(t, expected, workerLogURL)
}
