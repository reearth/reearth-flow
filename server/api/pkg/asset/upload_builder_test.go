package asset

import (
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/stretchr/testify/assert"
)

func TestUploadBuilder_NewUpload(t *testing.T) {
	tests := []struct {
		name string
		want *UploadBuilder
	}{
		{
			name: "success",
			want: &UploadBuilder{
				u: &Upload{},
			},
		},
	}
	for _, test := range tests {
		t.Run(string(test.name), func(t *testing.T) {
			t.Parallel()
			result := NewUpload()
			assert.Equal(t, test.want, result)
		})
	}
}

func TestUploadBuilderUUID(t *testing.T) {

	tests := []struct {
		name  string
		input string
		want  *UploadBuilder
	}{
		{
			name:  "success",
			input: "123",
			want: &UploadBuilder{
				&Upload{
					uuid: "123",
				},
			},
		},
	}
	for _, test := range tests {
		t.Run(string(test.name), func(t *testing.T) {
			t.Parallel()
			ub := NewUpload()
			result := ub.UUID(test.input)
			assert.Equal(t, test.want, result)
		})
	}
}

func TestUploadBuilderWorkspace(t *testing.T) {
	wid := accountsid.NewWorkspaceID()
	tests := []struct {
		name  string
		input accountsid.WorkspaceID
		want  *UploadBuilder
	}{
		{
			name:  "success",
			input: wid,
			want: &UploadBuilder{
				&Upload{
					workspace: wid,
				},
			},
		},
	}
	for _, test := range tests {
		t.Run(string(test.name), func(t *testing.T) {
			t.Parallel()
			ub := NewUpload()
			result := ub.Workspace(test.input)
			assert.Equal(t, test.want, result)
		})
	}
}

func TestUploadBuilderFileName(t *testing.T) {
	tests := []struct {
		name  string
		input string
		want  *UploadBuilder
	}{
		{
			name:  "success",
			input: "file.test",
			want: &UploadBuilder{
				&Upload{
					fileName: "file.test",
				},
			},
		},
	}
	for _, test := range tests {
		t.Run(string(test.name), func(t *testing.T) {
			t.Parallel()
			ub := NewUpload()
			result := ub.FileName(test.input)
			assert.Equal(t, test.want, result)
		})
	}
}

func TestUploadBuilderContentLength(t *testing.T) {
	tests := []struct {
		name  string
		input int64
		want  *UploadBuilder
	}{
		{
			name:  "success",
			input: 2,
			want: &UploadBuilder{
				&Upload{
					contentLength: 2,
				},
			},
		},
	}
	for _, test := range tests {
		t.Run(string(test.name), func(t *testing.T) {
			t.Parallel()
			ub := NewUpload()
			result := ub.ContentLength(test.input)
			assert.Equal(t, test.want, result)
		})
	}
}

func TestUploadBuilderExpiresAt(t *testing.T) {
	fixedTime := time.Date(2024, 1, 1, 0, 0, 0, 0, time.UTC)
	tests := []struct {
		name  string
		input time.Time
		want  *UploadBuilder
	}{
		{
			name:  "success",
			input: fixedTime,
			want: &UploadBuilder{
				&Upload{
					expiresAt: fixedTime,
				},
			},
		},
	}
	for _, test := range tests {
		t.Run(string(test.name), func(t *testing.T) {
			t.Parallel()
			ub := NewUpload()
			result := ub.ExpiresAt(test.input)
			assert.Equal(t, test.want, result)
		})
	}
}

func TestUploadBuilderBuild(t *testing.T) {
	now := time.Now()
	wid := accountsid.NewWorkspaceID()
	ubWithData := &UploadBuilder{
		u: &Upload{
			uuid:          "1",
			workspace:     wid,
			fileName:      "file.test",
			contentLength: int64(1),
			expiresAt:     now,
		},
	}

	tests := []struct {
		name  string
		input time.Time
		want  *Upload
	}{
		{
			name:  "success",
			input: now,
			want: &Upload{
				uuid:          "1",
				workspace:     wid,
				fileName:      "file.test",
				contentLength: int64(1),
				expiresAt:     now,
			},
		},
	}
	for _, test := range tests {
		t.Run(string(test.name), func(t *testing.T) {
			t.Parallel()
			result := ubWithData.Build()
			assert.Equal(t, test.want, result)
		})
	}
}
