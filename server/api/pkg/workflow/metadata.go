package workflow

import "time"

type Asset struct {
	BaseUrl string   `json:"baseUrl"`
	Files   []string `json:"files"`
}

type Timestamp struct {
	Created time.Time  `json:"created"`
	Updated *time.Time `json:"updated,omitempty"`
}

type Metadata struct {
	Timestamps      Timestamp `json:"timestamps"`
	Version         *string   `json:"version,omitempty"`
	ArtifactBaseUrl string    `json:"artifactBaseUrl"`
	JobID           string    `json:"jobId"`
	Assets          Asset     `json:"assets"`
	Tags            []string  `json:"tags,omitempty"`
}
