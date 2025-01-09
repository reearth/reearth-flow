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
	ArtifactBaseUrl string    `json:"artifactBaseUrl"`
	Assets          Asset     `json:"assets"`
	JobID           string    `json:"jobId"`
	Tags            []string  `json:"tags,omitempty"`
	Timestamps      Timestamp `json:"timestamps"`
	Version         *string   `json:"version,omitempty"`
}
