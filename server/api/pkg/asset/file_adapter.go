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

func DetectPreviewTypeFromFile(f *flowfile.File) *PreviewType {
	rxFile := ConvertFileToReearthx(f)
	rxPT := DetectPreviewType(rxFile)
	if rxPT == nil {
		return nil
	}
	pt := PreviewType(*rxPT)
	return &pt
}
