package fs

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/url"
	"os"
	"path"
	"path/filepath"
	"time"

	"github.com/kennygrant/sanitize"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
	"github.com/reearth/reearthx/rerror"
	"github.com/spf13/afero"
)

type fileRepo struct {
	fs              afero.Fs
	assetUrlBase    *url.URL
	workflowUrlBase *url.URL
}

func (f *fileRepo) CheckIntermediateDataExists(context.Context, string, string) (bool, error) {
	panic("unimplemented")
}

func (f *fileRepo) GetIntermediateDataURL(context.Context, string, string) string {
	panic("unimplemented")
}

func (f *fileRepo) CheckJobLogExists(ctx context.Context, jobID string) (bool, error) {
	logPath := filepath.Join(metadataDir, fmt.Sprintf("job-%s.log", jobID))
	exists, err := afero.Exists(f.fs, logPath)
	if err != nil {
		return false, rerror.ErrInternalByWithContext(ctx, err)
	}
	return exists, nil
}

func (f *fileRepo) GetJobLogURL(jobID string) string {
	return fmt.Sprintf("file://%s/job-%s.log", metadataDir, jobID)
}

func (f *fileRepo) GetJobWorkerLogURL(jobID string) string {
	return fmt.Sprintf("file://%s/job-%s-worker.log", metadataDir, jobID)
}

func (f *fileRepo) ListJobArtifacts(ctx context.Context, jobID string) ([]string, error) {
	artifactsPath := filepath.Join(metadataDir, fmt.Sprintf("job-%s-artifacts", jobID))
	files, err := afero.ReadDir(f.fs, artifactsPath)
	if err != nil {
		if os.IsNotExist(err) {
			return []string{}, nil
		}
		return nil, rerror.ErrInternalByWithContext(ctx, err)
	}

	var artifacts []string
	for _, file := range files {
		artifacts = append(artifacts, file.Name())
	}
	return artifacts, nil
}

func (f *fileRepo) ReadArtifact(ctx context.Context, path string) (io.ReadCloser, error) {
	return f.read(ctx, path)
}

func (f *fileRepo) ReadMetadata(ctx context.Context, name string) (io.ReadCloser, error) {
	return f.read(ctx, filepath.Join(metadataDir, sanitize.Path(name)))
}

func (f *fileRepo) RemoveMetadata(ctx context.Context, u *url.URL) error {
	if u == nil {
		return nil
	}
	p := sanitize.Path(u.Path)
	if p == "" || !f.validateURL(u, f.workflowUrlBase) {
		return gateway.ErrInvalidFile
	}
	return f.delete(ctx, filepath.Join(metadataDir, filepath.Base(p)))
}

func (f *fileRepo) UploadMetadata(ctx context.Context, jobID string, assets []string) (*url.URL, error) {
	metadataFile, err := f.generateMetadata(jobID, assets)
	if err != nil {
		return nil, err
	}

	filename := filepath.Join(metadataDir, sanitize.Path(metadataFile.Path))
	_, err = f.upload(ctx, filename, metadataFile.Content)
	if err != nil {
		return nil, err
	}

	return getFileURL(f.workflowUrlBase, metadataFile.Path), nil
}

func (f *fileRepo) generateMetadata(jobID string, assets []string) (*file.File, error) {
	artifactBaseUrl := "file://artifacts"
	assetBaseUrl := "file://assets"
	created := time.Now()

	metadata := &workflow.Metadata{
		ArtifactBaseUrl: artifactBaseUrl,
		Assets: workflow.Asset{
			BaseUrl: assetBaseUrl,
			Files:   assets,
		},
		JobID: jobID,
		Timestamps: workflow.Timestamp{
			Created: created,
		},
	}

	metadataJSON, err := json.Marshal(metadata)
	if err != nil {
		return nil, err
	}

	return &file.File{
		Content:     io.NopCloser(bytes.NewReader(metadataJSON)),
		Path:        fmt.Sprintf("metadata-%s.json", jobID),
		Size:        int64(len(metadataJSON)),
		ContentType: "application/json",
	}, nil
}

func NewFile(fs afero.Fs, assetUrlBase string, workflowUrlBase string) (gateway.File, error) {
	var err error
	aurlb, err := url.Parse(assetUrlBase)
	if err != nil {
		return nil, errors.New("invalid base URL")
	}
	wurlb, err := url.Parse(workflowUrlBase)
	if err != nil {
		return nil, errors.New("invalid base URL")
	}

	return &fileRepo{
		fs:              fs,
		assetUrlBase:    aurlb,
		workflowUrlBase: wurlb,
	}, nil
}

func (f *fileRepo) ReadAsset(ctx context.Context, filename string) (io.ReadCloser, error) {
	return f.read(ctx, filepath.Join(assetDir, sanitize.Path(filename)))
}

func (f *fileRepo) UploadAsset(ctx context.Context, file *file.File) (*url.URL, int64, error) {
	filename := sanitize.Path(newAssetID() + filepath.Ext(file.Path))
	size, err := f.upload(ctx, filepath.Join(assetDir, filename), file.Content)
	if err != nil {
		return nil, 0, err
	}
	return getFileURL(f.assetUrlBase, filename), size, nil
}

func (f *fileRepo) RemoveAsset(ctx context.Context, u *url.URL) error {
	if u == nil {
		return nil
	}
	p := sanitize.Path(u.Path)
	if p == "" || !f.validateURL(u, f.assetUrlBase) {
		return gateway.ErrInvalidFile
	}
	return f.delete(ctx, filepath.Join(assetDir, filepath.Base(p)))
}

func (f *fileRepo) ReadWorkflow(ctx context.Context, filename string) (io.ReadCloser, error) {
	return f.read(ctx, filepath.Join(workflowsDir, sanitize.Path(filename)))
}

func (f *fileRepo) UploadWorkflow(ctx context.Context, file *file.File) (*url.URL, error) {
	filename := sanitize.Path(newWorkflowID() + filepath.Ext(file.Path))
	_, err := f.upload(ctx, filepath.Join(workflowsDir, filename), file.Content)
	if err != nil {
		return nil, err
	}
	return getFileURL(f.workflowUrlBase, filename), nil
}

func (f *fileRepo) RemoveWorkflow(ctx context.Context, u *url.URL) error {
	if u == nil {
		return nil
	}
	p := sanitize.Path(u.Path)
	if p == "" || !f.validateURL(u, f.workflowUrlBase) {
		return gateway.ErrInvalidFile
	}
	return f.delete(ctx, filepath.Join(workflowsDir, filepath.Base(p)))
}

func (f *fileRepo) CheckJobWorkerLogExists(ctx context.Context, jobID string) (bool, error) {
	logPath := filepath.Join(metadataDir, fmt.Sprintf("job-%s-worker.log", jobID))
	exists, err := afero.Exists(f.fs, logPath)
	if err != nil {
		return false, rerror.ErrInternalByWithContext(ctx, err)
	}
	return exists, nil
}

// helpers

func (f *fileRepo) read(ctx context.Context, filename string) (io.ReadCloser, error) {
	if filename == "" {
		return nil, rerror.ErrNotFound
	}

	file, err := f.fs.Open(filename)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, rerror.ErrNotFound
		}
		return nil, rerror.ErrInternalByWithContext(ctx, err)
	}
	return file, nil
}

func (f *fileRepo) upload(ctx context.Context, filename string, content io.Reader) (int64, error) {
	if filename == "" {
		return 0, gateway.ErrFailedToUploadFile
	}

	if fnd := filepath.Dir(filename); fnd != "" {
		if err := f.fs.MkdirAll(fnd, 0755); err != nil {
			return 0, rerror.ErrInternalByWithContext(ctx, err)
		}
	}

	dest, err := f.fs.Create(filename)
	if err != nil {
		return 0, rerror.ErrInternalByWithContext(ctx, err)
	}
	defer func() {
		_ = dest.Close()
	}()

	size, err := io.Copy(dest, content)
	if err != nil {
		return 0, gateway.ErrFailedToUploadFile
	}

	return size, nil
}

func (f *fileRepo) delete(ctx context.Context, filename string) error {
	if filename == "" {
		return gateway.ErrFailedToUploadFile
	}

	if err := f.fs.RemoveAll(filename); err != nil {
		if os.IsNotExist(err) {
			return nil
		}
		return rerror.ErrInternalByWithContext(ctx, err)
	}
	return nil
}

func getFileURL(base *url.URL, filename string) *url.URL {
	if base == nil {
		return nil
	}

	// https://github.com/golang/go/issues/38351
	b := *base
	b.Path = path.Join(b.Path, filename)
	return &b
}

func newAssetID() string {
	return id.NewAssetID().String()
}

func newWorkflowID() string {
	return id.NewWorkflowID().String()
}

func (f *fileRepo) validateURL(u *url.URL, base *url.URL) bool {
	if u == nil || base == nil {
		return false
	}
	
	// Handle the case where base path is empty (e.g., https://example.com)
	basePath := base.Path
	if basePath == "" {
		basePath = "/"
	}
	
	return u.Scheme == base.Scheme &&
		u.Host == base.Host &&
		path.Dir(u.Path) == basePath
}
