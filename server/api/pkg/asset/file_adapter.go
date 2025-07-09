package asset

import (
	flowfile "github.com/reearth/reearth-flow/api/pkg/file"
	reearthxfile "github.com/reearth/reearthx/asset/domain/file"
)

func ConvertFileToReearthx(f *flowfile.File) *reearthxfile.File {
	if f == nil {
		return nil
	}
	return &reearthxfile.File{
		Content:     f.Content,
		Path:        f.Path,
		Size:        f.Size,
		ContentType: f.ContentType,
	}
}
