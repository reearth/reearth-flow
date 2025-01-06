package gcs

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/url"
	"path"
	"strings"
	"time"

	"cloud.google.com/go/storage"
	"github.com/kennygrant/sanitize"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/file"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workflow"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/rerror"
)

const (
	gcsAssetBasePath    string = "assets"
	gcsWorkflowBasePath string = "workflows"
	gcsMetadataBasePath string = "metadata"
	fileSizeLimit       int64  = 1024 * 1024 * 100 // about 100MB
)

type fileRepo struct {
	bucketName   string
	base         *url.URL
	cacheControl string
}

func NewFile(bucketName, base string, cacheControl string) (gateway.File, error) {
	if bucketName == "" {
		return nil, errors.New("bucket name is empty")
	}

	var u *url.URL
	if base == "" {
		base = fmt.Sprintf("https://storage.googleapis.com/%s", bucketName)
	}

	var err error
	u, err = url.Parse(base)
	if err != nil {
		return nil, errors.New("invalid base URL")
	}

	return &fileRepo{
		bucketName:   bucketName,
		base:         u,
		cacheControl: cacheControl,
	}, nil
}

func (f *fileRepo) ReadAsset(ctx context.Context, name string) (io.ReadCloser, error) {
	sn := sanitize.Path(name)
	if sn == "" {
		return nil, rerror.ErrNotFound
	}
	return f.read(ctx, path.Join(gcsAssetBasePath, sn))
}

func (f *fileRepo) UploadAsset(ctx context.Context, file *file.File) (*url.URL, int64, error) {
	if file == nil {
		return nil, 0, gateway.ErrInvalidFile
	}
	if file.Size >= fileSizeLimit {
		return nil, 0, gateway.ErrFileTooLarge
	}

	sn := sanitize.Path(newAssetID() + path.Ext(file.Path))
	if sn == "" {
		return nil, 0, gateway.ErrInvalidFile
	}

	filename := path.Join(gcsAssetBasePath, sn)
	u := getGCSObjectURL(f.base, filename)
	if u == nil {
		return nil, 0, gateway.ErrInvalidFile
	}

	s, err := f.upload(ctx, filename, file.Content)
	if err != nil {
		return nil, 0, err
	}
	return u, s, nil
}

func (f *fileRepo) RemoveAsset(ctx context.Context, u *url.URL) error {
	log.Infofc(ctx, "gcs: asset deleted: %s", u)

	sn := getGCSObjectNameFromURL(f.base, u, gcsAssetBasePath)
	if sn == "" {
		return gateway.ErrInvalidFile
	}
	return f.delete(ctx, sn)
}

func (f *fileRepo) ReadWorkflow(ctx context.Context, name string) (io.ReadCloser, error) {
	sn := sanitize.Path(name)
	if sn == "" {
		return nil, rerror.ErrNotFound
	}
	return f.read(ctx, path.Join(gcsWorkflowBasePath, sn))
}

func (f *fileRepo) UploadWorkflow(ctx context.Context, file *file.File) (*url.URL, error) {
	if file == nil {
		return nil, gateway.ErrInvalidFile
	}

	sn := sanitize.Path(newWorkflowID() + path.Ext(file.Path))
	if sn == "" {
		return nil, gateway.ErrInvalidFile
	}

	filename := path.Join(gcsWorkflowBasePath, sn)
	u := getGCSObjectURL(f.base, filename)
	if u == nil {
		return nil, gateway.ErrInvalidFile
	}

	_, err := f.upload(ctx, filename, file.Content)
	if err != nil {
		return nil, err
	}
	return u, nil
}

func (f *fileRepo) RemoveWorkflow(ctx context.Context, u *url.URL) error {
	log.Infofc(ctx, "gcs: workflow deleted: %s", u)

	sn := getGCSObjectNameFromURL(f.base, u, gcsWorkflowBasePath)
	if sn == "" {
		return gateway.ErrInvalidFile
	}
	return f.delete(ctx, sn)
}

func (f *fileRepo) ReadMetadata(ctx context.Context, name string) (io.ReadCloser, error) {
	sn := sanitize.Path(name)
	if sn == "" {
		return nil, rerror.ErrNotFound
	}
	return f.read(ctx, path.Join(gcsMetadataBasePath, sn))
}

func (f *fileRepo) UploadMetadata(ctx context.Context, jobID string, assets []string) (*url.URL, error) {
	metadataFile, err := f.generateMetadata(jobID, assets)
	if err != nil {
		return nil, err
	}

	sn := sanitize.Path(metadataFile.Path)
	if sn == "" {
		return nil, gateway.ErrInvalidFile
	}

	filename := path.Join(gcsMetadataBasePath, sn)
	u := getGCSObjectURL(f.base, filename)
	if u == nil {
		return nil, gateway.ErrInvalidFile
	}

	_, err = f.upload(ctx, filename, metadataFile.Content)
	if err != nil {
		return nil, err
	}

	log.Infofc(ctx, "gcs: metadata uploaded: %s with jobID: %s and %d assets", u, jobID, len(assets))
	return u, nil
}

func (f *fileRepo) RemoveMetadata(ctx context.Context, u *url.URL) error {
	log.Infofc(ctx, "gcs: metadata deleted: %s", u)

	sn := getGCSObjectNameFromURL(f.base, u, gcsMetadataBasePath)
	if sn == "" {
		return gateway.ErrInvalidFile
	}
	return f.delete(ctx, sn)
}

// helpers
func (f *fileRepo) bucket(ctx context.Context) (*storage.BucketHandle, error) {
	client, err := storage.NewClient(ctx)
	if err != nil {
		return nil, err
	}
	bucket := client.Bucket(f.bucketName)
	return bucket, nil
}

func (f *fileRepo) read(ctx context.Context, filename string) (io.ReadCloser, error) {
	if filename == "" {
		return nil, rerror.ErrNotFound
	}

	bucket, err := f.bucket(ctx)
	if err != nil {
		log.Errorfc(ctx, "gcs: read bucket err: %+v\n", err)
		return nil, rerror.ErrInternalByWithContext(ctx, err)
	}

	reader, err := bucket.Object(filename).NewReader(ctx)
	if err != nil {
		if errors.Is(err, storage.ErrObjectNotExist) {
			return nil, rerror.ErrNotFound
		}
		log.Errorfc(ctx, "gcs: read err: %+v\n", err)
		return nil, rerror.ErrInternalByWithContext(ctx, err)
	}

	return reader, nil
}

func (f *fileRepo) upload(ctx context.Context, filename string, content io.Reader) (int64, error) {
	if filename == "" {
		return 0, gateway.ErrInvalidFile
	}

	bucket, err := f.bucket(ctx)
	if err != nil {
		log.Errorfc(ctx, "gcs: upload bucket err: %+v\n", err)
		return 0, rerror.ErrInternalByWithContext(ctx, err)
	}

	object := bucket.Object(filename)
	if err := object.Delete(ctx); err != nil && !errors.Is(err, storage.ErrObjectNotExist) {
		log.Errorfc(ctx, "gcs: upload delete err: %+v\n", err)
		return 0, gateway.ErrFailedToUploadFile
	}

	writer := object.NewWriter(ctx)
	writer.ObjectAttrs.CacheControl = f.cacheControl

	size, err := io.Copy(writer, content)
	if err != nil {
		log.Errorfc(ctx, "gcs: upload err: %+v\n", err)
		return 0, gateway.ErrFailedToUploadFile
	}

	if err := writer.Close(); err != nil {
		log.Errorfc(ctx, "gcs: upload close err: %+v\n", err)
		return 0, gateway.ErrFailedToUploadFile
	}

	return size, nil
}

func (f *fileRepo) delete(ctx context.Context, filename string) error {
	if filename == "" {
		return gateway.ErrInvalidFile
	}

	bucket, err := f.bucket(ctx)
	if err != nil {
		log.Errorfc(ctx, "gcs: delete bucket err: %+v\n", err)
		return rerror.ErrInternalByWithContext(ctx, err)
	}

	object := bucket.Object(filename)
	if err := object.Delete(ctx); err != nil {
		if errors.Is(err, storage.ErrObjectNotExist) {
			return nil
		}

		log.Errorfc(ctx, "gcs: delete err: %+v\n", err)
		return rerror.ErrInternalByWithContext(ctx, err)
	}
	return nil
}

func (f *fileRepo) generateMetadata(jobID string, assets []string) (*file.File, error) {
	artifactBaseUrl := fmt.Sprintf("gs://%s/artifacts", f.bucketName)
	assetBaseUrl := fmt.Sprintf("gs://%s/assets", f.bucketName)
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

func getGCSObjectURL(base *url.URL, objectName string) *url.URL {
	if base == nil {
		return nil
	}

	// https://github.com/golang/go/issues/38351
	b := *base
	b.Path = path.Join(b.Path, objectName)
	return &b
}

func getGCSObjectNameFromURL(base, u *url.URL, gcsBasePath string) string {
	if u == nil {
		return ""
	}
	if base == nil {
		base = &url.URL{}
	}
	p := sanitize.Path(strings.TrimPrefix(u.Path, "/"))
	if p == "" || u.Host != base.Host || u.Scheme != base.Scheme || !strings.HasPrefix(p, gcsBasePath+"/") {
		return ""
	}

	return p
}

func newAssetID() string {
	return id.NewAssetID().String()
}

func newWorkflowID() string {
	return id.NewWorkflowID().String()
}
