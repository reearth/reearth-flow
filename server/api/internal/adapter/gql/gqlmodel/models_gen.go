// Code generated by github.com/99designs/gqlgen, DO NOT EDIT.

package gqlmodel

import (
	"bytes"
	"fmt"
	"io"
	"strconv"
	"time"

	"github.com/99designs/gqlgen/graphql"
	"golang.org/x/text/language"
)

type Node interface {
	IsNode()
	GetID() ID
}

type APIDriverInput struct {
	Token string `json:"token"`
}

type AddMemberToWorkspaceInput struct {
	WorkspaceID ID   `json:"workspaceId"`
	UserID      ID   `json:"userId"`
	Role        Role `json:"role"`
}

type AddMemberToWorkspacePayload struct {
	Workspace *Workspace `json:"workspace"`
}

type Asset struct {
	ID                      ID                       `json:"id"`
	WorkspaceID             ID                       `json:"workspaceId"`
	CreatedAt               time.Time                `json:"createdAt"`
	FileName                string                   `json:"fileName"`
	Size                    int64                    `json:"size"`
	ContentType             string                   `json:"contentType"`
	Name                    string                   `json:"name"`
	URL                     string                   `json:"url"`
	UUID                    string                   `json:"uuid"`
	FlatFiles               bool                     `json:"flatFiles"`
	Public                  bool                     `json:"public"`
	ArchiveExtractionStatus *ArchiveExtractionStatus `json:"archiveExtractionStatus,omitempty"`
	Workspace               *Workspace               `json:"Workspace,omitempty"`
}

func (Asset) IsNode()        {}
func (this Asset) GetID() ID { return this.ID }

type AssetConnection struct {
	Nodes      []*Asset  `json:"nodes"`
	PageInfo   *PageInfo `json:"pageInfo"`
	TotalCount int       `json:"totalCount"`
}

type CancelJobInput struct {
	JobID ID `json:"jobId"`
}

type CancelJobPayload struct {
	Job *Job `json:"job,omitempty"`
}

type CreateAssetInput struct {
	WorkspaceID ID             `json:"workspaceId"`
	File        graphql.Upload `json:"file"`
	Name        *string        `json:"name,omitempty"`
}

type CreateAssetPayload struct {
	Asset *Asset `json:"asset"`
}

type CreateDeploymentInput struct {
	WorkspaceID ID             `json:"workspaceId"`
	File        graphql.Upload `json:"file"`
	ProjectID   *ID            `json:"projectId,omitempty"`
	Description string         `json:"description"`
}

type CreateProjectInput struct {
	WorkspaceID ID      `json:"workspaceId"`
	Name        *string `json:"name,omitempty"`
	Description *string `json:"description,omitempty"`
	Archived    *bool   `json:"archived,omitempty"`
}

type CreateTriggerInput struct {
	WorkspaceID     ID               `json:"workspaceId"`
	DeploymentID    ID               `json:"deploymentId"`
	Description     string           `json:"description"`
	TimeDriverInput *TimeDriverInput `json:"timeDriverInput,omitempty"`
	APIDriverInput  *APIDriverInput  `json:"apiDriverInput,omitempty"`
}

type CreateWorkspaceInput struct {
	Name string `json:"name"`
}

type CreateWorkspacePayload struct {
	Workspace *Workspace `json:"workspace"`
}

type DeclareParameterInput struct {
	Name         string        `json:"name"`
	Type         ParameterType `json:"type"`
	Required     bool          `json:"required"`
	Public       bool          `json:"public"`
	DefaultValue any           `json:"defaultValue,omitempty"`
	Config       JSON          `json:"config,omitempty"`
	Index        *int          `json:"index,omitempty"`
}

type DeleteAssetInput struct {
	AssetID ID `json:"assetId"`
}

type DeleteAssetPayload struct {
	AssetID ID `json:"assetId"`
}

type DeleteDeploymentInput struct {
	DeploymentID ID `json:"deploymentId"`
}

type DeleteDeploymentPayload struct {
	DeploymentID ID `json:"deploymentId"`
}

type DeleteMeInput struct {
	UserID ID `json:"userId"`
}

type DeleteMePayload struct {
	UserID ID `json:"userId"`
}

type DeleteProjectInput struct {
	ProjectID ID `json:"projectId"`
}

type DeleteProjectPayload struct {
	ProjectID ID `json:"projectId"`
}

type DeleteWorkspaceInput struct {
	WorkspaceID ID `json:"workspaceId"`
}

type DeleteWorkspacePayload struct {
	WorkspaceID ID `json:"workspaceId"`
}

type Deployment struct {
	CreatedAt   time.Time  `json:"createdAt"`
	Description string     `json:"description"`
	HeadID      *ID        `json:"headId,omitempty"`
	IsHead      bool       `json:"isHead"`
	ID          ID         `json:"id"`
	Project     *Project   `json:"project,omitempty"`
	ProjectID   *ID        `json:"projectId,omitempty"`
	UpdatedAt   time.Time  `json:"updatedAt"`
	Version     string     `json:"version"`
	WorkflowURL string     `json:"workflowUrl"`
	Workspace   *Workspace `json:"workspace,omitempty"`
	WorkspaceID ID         `json:"workspaceId"`
}

func (Deployment) IsNode()        {}
func (this Deployment) GetID() ID { return this.ID }

type DeploymentConnection struct {
	Nodes      []*Deployment `json:"nodes"`
	PageInfo   *PageInfo     `json:"pageInfo"`
	TotalCount int           `json:"totalCount"`
}

type DeploymentPayload struct {
	Deployment *Deployment `json:"deployment"`
}

type ExecuteDeploymentInput struct {
	DeploymentID ID `json:"deploymentId"`
}

type GetByVersionInput struct {
	WorkspaceID ID     `json:"workspaceId"`
	ProjectID   *ID    `json:"projectId,omitempty"`
	Version     string `json:"version"`
}

type GetHeadInput struct {
	WorkspaceID ID  `json:"workspaceId"`
	ProjectID   *ID `json:"projectId,omitempty"`
}

type Job struct {
	CompletedAt   *time.Time  `json:"completedAt,omitempty"`
	Deployment    *Deployment `json:"deployment,omitempty"`
	DeploymentID  ID          `json:"deploymentId"`
	Debug         *bool       `json:"debug,omitempty"`
	ID            ID          `json:"id"`
	LogsURL       *string     `json:"logsURL,omitempty"`
	WorkerLogsURL *string     `json:"workerLogsURL,omitempty"`
	OutputURLs    []string    `json:"outputURLs,omitempty"`
	StartedAt     time.Time   `json:"startedAt"`
	Status        JobStatus   `json:"status"`
	Workspace     *Workspace  `json:"workspace,omitempty"`
	WorkspaceID   ID          `json:"workspaceId"`
	Logs          []*Log      `json:"logs,omitempty"`
}

func (Job) IsNode()        {}
func (this Job) GetID() ID { return this.ID }

type JobConnection struct {
	Nodes      []*Job    `json:"nodes"`
	PageInfo   *PageInfo `json:"pageInfo"`
	TotalCount int       `json:"totalCount"`
}

type JobPayload struct {
	Job *Job `json:"job"`
}

type Log struct {
	JobID     ID        `json:"jobId"`
	NodeID    *ID       `json:"nodeId,omitempty"`
	Timestamp time.Time `json:"timestamp"`
	LogLevel  LogLevel  `json:"logLevel"`
	Message   string    `json:"message"`
}

type Me struct {
	Auths         []string     `json:"auths"`
	Email         string       `json:"email"`
	ID            ID           `json:"id"`
	Lang          language.Tag `json:"lang"`
	MyWorkspace   *Workspace   `json:"myWorkspace,omitempty"`
	MyWorkspaceID ID           `json:"myWorkspaceId"`
	Name          string       `json:"name"`
	Workspaces    []*Workspace `json:"workspaces"`
}

type Mutation struct {
}

type NodeExecution struct {
	ID          ID         `json:"id"`
	JobID       ID         `json:"jobId"`
	NodeID      ID         `json:"nodeId"`
	Status      NodeStatus `json:"status"`
	CreatedAt   *time.Time `json:"createdAt,omitempty"`
	StartedAt   *time.Time `json:"startedAt,omitempty"`
	CompletedAt *time.Time `json:"completedAt,omitempty"`
}

func (NodeExecution) IsNode()        {}
func (this NodeExecution) GetID() ID { return this.ID }

type PageBasedPagination struct {
	Page     int             `json:"page"`
	PageSize int             `json:"pageSize"`
	OrderBy  *string         `json:"orderBy,omitempty"`
	OrderDir *OrderDirection `json:"orderDir,omitempty"`
}

type PageInfo struct {
	TotalCount  int  `json:"totalCount"`
	CurrentPage *int `json:"currentPage,omitempty"`
	TotalPages  *int `json:"totalPages,omitempty"`
}

type Pagination struct {
	Page     *int            `json:"page,omitempty"`
	PageSize *int            `json:"pageSize,omitempty"`
	OrderBy  *string         `json:"orderBy,omitempty"`
	OrderDir *OrderDirection `json:"orderDir,omitempty"`
}

type Parameter struct {
	CreatedAt    time.Time     `json:"createdAt"`
	ID           ID            `json:"id"`
	Index        int           `json:"index"`
	Name         string        `json:"name"`
	ProjectID    ID            `json:"projectId"`
	Required     bool          `json:"required"`
	Public       bool          `json:"public"`
	Type         ParameterType `json:"type"`
	UpdatedAt    time.Time     `json:"updatedAt"`
	DefaultValue any           `json:"defaultValue"`
	Config       JSON          `json:"config,omitempty"`
}

type ParameterBatchInput struct {
	ProjectID ID                           `json:"projectId"`
	Creates   []*DeclareParameterInput     `json:"creates,omitempty"`
	Updates   []*ParameterUpdateItem       `json:"updates,omitempty"`
	Deletes   []ID                         `json:"deletes,omitempty"`
	Reorders  []*UpdateParameterOrderInput `json:"reorders,omitempty"`
}

type ParameterUpdateItem struct {
	ParamID      ID             `json:"paramId"`
	Name         *string        `json:"name,omitempty"`
	Type         *ParameterType `json:"type,omitempty"`
	Required     *bool          `json:"required,omitempty"`
	Public       *bool          `json:"public,omitempty"`
	DefaultValue any            `json:"defaultValue,omitempty"`
	Config       JSON           `json:"config,omitempty"`
}

type PreviewSnapshot struct {
	ID        ID        `json:"id"`
	Name      *string   `json:"name,omitempty"`
	Timestamp time.Time `json:"timestamp"`
	Updates   []int     `json:"updates"`
	Version   int       `json:"version"`
}

type Project struct {
	BasicAuthPassword string       `json:"basicAuthPassword"`
	BasicAuthUsername string       `json:"basicAuthUsername"`
	CreatedAt         time.Time    `json:"createdAt"`
	Description       string       `json:"description"`
	Deployment        *Deployment  `json:"deployment,omitempty"`
	ID                ID           `json:"id"`
	IsArchived        bool         `json:"isArchived"`
	IsBasicAuthActive bool         `json:"isBasicAuthActive"`
	Name              string       `json:"name"`
	Parameters        []*Parameter `json:"parameters"`
	UpdatedAt         time.Time    `json:"updatedAt"`
	SharedToken       *string      `json:"sharedToken,omitempty"`
	Version           int          `json:"version"`
	Workspace         *Workspace   `json:"workspace,omitempty"`
	WorkspaceID       ID           `json:"workspaceId"`
}

func (Project) IsNode()        {}
func (this Project) GetID() ID { return this.ID }

type ProjectConnection struct {
	Nodes      []*Project `json:"nodes"`
	PageInfo   *PageInfo  `json:"pageInfo"`
	TotalCount int        `json:"totalCount"`
}

type ProjectDocument struct {
	ID        ID        `json:"id"`
	Timestamp time.Time `json:"timestamp"`
	Updates   []int     `json:"updates"`
	Version   int       `json:"version"`
}

func (ProjectDocument) IsNode()        {}
func (this ProjectDocument) GetID() ID { return this.ID }

type ProjectPayload struct {
	Project *Project `json:"project"`
}

type ProjectSharingInfoPayload struct {
	ProjectID    ID      `json:"projectId"`
	SharingToken *string `json:"sharingToken,omitempty"`
}

type ProjectSnapshot struct {
	Timestamp time.Time `json:"timestamp"`
	Updates   []int     `json:"updates"`
	Version   int       `json:"version"`
}

type ProjectSnapshotMetadata struct {
	Timestamp time.Time `json:"timestamp"`
	Version   int       `json:"version"`
}

type Query struct {
}

type RemoveMemberFromWorkspaceInput struct {
	WorkspaceID ID `json:"workspaceId"`
	UserID      ID `json:"userId"`
}

type RemoveMemberFromWorkspacePayload struct {
	Workspace *Workspace `json:"workspace"`
}

type RemoveMyAuthInput struct {
	Auth string `json:"auth"`
}

type RemoveParameterInput struct {
	ParamID ID `json:"paramId"`
}

type RemoveParametersInput struct {
	ParamIds []ID `json:"paramIds"`
}

type RunProjectInput struct {
	ProjectID   ID             `json:"projectId"`
	WorkspaceID ID             `json:"workspaceId"`
	File        graphql.Upload `json:"file"`
}

type RunProjectPayload struct {
	Job *Job `json:"job"`
}

type ShareProjectInput struct {
	ProjectID ID `json:"projectId"`
}

type ShareProjectPayload struct {
	ProjectID  ID     `json:"projectId"`
	SharingURL string `json:"sharingUrl"`
}

type SharedProjectPayload struct {
	Project *Project `json:"project"`
}

type SignupInput struct {
	UserID      *ID           `json:"userId,omitempty"`
	Lang        *language.Tag `json:"lang,omitempty"`
	WorkspaceID *ID           `json:"workspaceId,omitempty"`
	Secret      *string       `json:"secret,omitempty"`
}

type SignupPayload struct {
	User      *User      `json:"user"`
	Workspace *Workspace `json:"workspace"`
}

type Subscription struct {
}

type TimeDriverInput struct {
	Interval TimeInterval `json:"interval"`
}

type Trigger struct {
	ID            ID              `json:"id"`
	CreatedAt     time.Time       `json:"createdAt"`
	UpdatedAt     time.Time       `json:"updatedAt"`
	LastTriggered *time.Time      `json:"lastTriggered,omitempty"`
	WorkspaceID   ID              `json:"workspaceId"`
	Workspace     *Workspace      `json:"workspace,omitempty"`
	Deployment    *Deployment     `json:"deployment"`
	DeploymentID  ID              `json:"deploymentId"`
	EventSource   EventSourceType `json:"eventSource"`
	Description   string          `json:"description"`
	AuthToken     *string         `json:"authToken,omitempty"`
	TimeInterval  *TimeInterval   `json:"timeInterval,omitempty"`
}

func (Trigger) IsNode()        {}
func (this Trigger) GetID() ID { return this.ID }

type TriggerConnection struct {
	Nodes      []*Trigger `json:"nodes"`
	PageInfo   *PageInfo  `json:"pageInfo"`
	TotalCount int        `json:"totalCount"`
}

type UnshareProjectInput struct {
	ProjectID ID `json:"projectId"`
}

type UnshareProjectPayload struct {
	ProjectID ID `json:"projectId"`
}

type UpdateAssetInput struct {
	AssetID ID      `json:"assetId"`
	Name    *string `json:"name,omitempty"`
}

type UpdateAssetPayload struct {
	Asset *Asset `json:"asset"`
}

type UpdateDeploymentInput struct {
	DeploymentID ID              `json:"deploymentId"`
	File         *graphql.Upload `json:"file,omitempty"`
	Description  *string         `json:"description,omitempty"`
}

type UpdateMeInput struct {
	Name                 *string       `json:"name,omitempty"`
	Email                *string       `json:"email,omitempty"`
	Password             *string       `json:"password,omitempty"`
	PasswordConfirmation *string       `json:"passwordConfirmation,omitempty"`
	Lang                 *language.Tag `json:"lang,omitempty"`
}

type UpdateMePayload struct {
	Me *Me `json:"me"`
}

type UpdateMemberOfWorkspaceInput struct {
	WorkspaceID ID   `json:"workspaceId"`
	UserID      ID   `json:"userId"`
	Role        Role `json:"role"`
}

type UpdateMemberOfWorkspacePayload struct {
	Workspace *Workspace `json:"workspace"`
}

type UpdateParameterInput struct {
	DefaultValue any           `json:"defaultValue"`
	Name         string        `json:"name"`
	Required     bool          `json:"required"`
	Public       bool          `json:"public"`
	Type         ParameterType `json:"type"`
	Config       JSON          `json:"config,omitempty"`
}

type UpdateParameterOrderInput struct {
	ParamID  ID  `json:"paramId"`
	NewIndex int `json:"newIndex"`
}

type UpdateProjectInput struct {
	ProjectID         ID      `json:"projectId"`
	Name              *string `json:"name,omitempty"`
	Description       *string `json:"description,omitempty"`
	Archived          *bool   `json:"archived,omitempty"`
	IsBasicAuthActive *bool   `json:"isBasicAuthActive,omitempty"`
	BasicAuthUsername *string `json:"basicAuthUsername,omitempty"`
	BasicAuthPassword *string `json:"basicAuthPassword,omitempty"`
}

type UpdateTriggerInput struct {
	TriggerID       ID               `json:"triggerId"`
	Description     *string          `json:"description,omitempty"`
	DeploymentID    *ID              `json:"deploymentId,omitempty"`
	TimeDriverInput *TimeDriverInput `json:"timeDriverInput,omitempty"`
	APIDriverInput  *APIDriverInput  `json:"apiDriverInput,omitempty"`
}

type UpdateWorkspaceInput struct {
	WorkspaceID ID     `json:"workspaceId"`
	Name        string `json:"name"`
}

type UpdateWorkspacePayload struct {
	Workspace *Workspace `json:"workspace"`
}

type User struct {
	Email string  `json:"email"`
	Host  *string `json:"host,omitempty"`
	ID    ID      `json:"id"`
	Name  string  `json:"name"`
}

func (User) IsNode()        {}
func (this User) GetID() ID { return this.ID }

type Workspace struct {
	Assets   *AssetConnection   `json:"assets"`
	ID       ID                 `json:"id"`
	Members  []*WorkspaceMember `json:"members"`
	Name     string             `json:"name"`
	Personal bool               `json:"personal"`
	Projects *ProjectConnection `json:"projects"`
}

func (Workspace) IsNode()        {}
func (this Workspace) GetID() ID { return this.ID }

type WorkspaceMember struct {
	Role   Role  `json:"role"`
	User   *User `json:"user,omitempty"`
	UserID ID    `json:"userId"`
}

type ArchiveExtractionStatus string

const (
	ArchiveExtractionStatusSkipped    ArchiveExtractionStatus = "SKIPPED"
	ArchiveExtractionStatusPending    ArchiveExtractionStatus = "PENDING"
	ArchiveExtractionStatusInProgress ArchiveExtractionStatus = "IN_PROGRESS"
	ArchiveExtractionStatusDone       ArchiveExtractionStatus = "DONE"
	ArchiveExtractionStatusFailed     ArchiveExtractionStatus = "FAILED"
)

var AllArchiveExtractionStatus = []ArchiveExtractionStatus{
	ArchiveExtractionStatusSkipped,
	ArchiveExtractionStatusPending,
	ArchiveExtractionStatusInProgress,
	ArchiveExtractionStatusDone,
	ArchiveExtractionStatusFailed,
}

func (e ArchiveExtractionStatus) IsValid() bool {
	switch e {
	case ArchiveExtractionStatusSkipped, ArchiveExtractionStatusPending, ArchiveExtractionStatusInProgress, ArchiveExtractionStatusDone, ArchiveExtractionStatusFailed:
		return true
	}
	return false
}

func (e ArchiveExtractionStatus) String() string {
	return string(e)
}

func (e *ArchiveExtractionStatus) UnmarshalGQL(v any) error {
	str, ok := v.(string)
	if !ok {
		return fmt.Errorf("enums must be strings")
	}

	*e = ArchiveExtractionStatus(str)
	if !e.IsValid() {
		return fmt.Errorf("%s is not a valid ArchiveExtractionStatus", str)
	}
	return nil
}

func (e ArchiveExtractionStatus) MarshalGQL(w io.Writer) {
	fmt.Fprint(w, strconv.Quote(e.String()))
}

func (e *ArchiveExtractionStatus) UnmarshalJSON(b []byte) error {
	s, err := strconv.Unquote(string(b))
	if err != nil {
		return err
	}
	return e.UnmarshalGQL(s)
}

func (e ArchiveExtractionStatus) MarshalJSON() ([]byte, error) {
	var buf bytes.Buffer
	e.MarshalGQL(&buf)
	return buf.Bytes(), nil
}

type AssetSortType string

const (
	AssetSortTypeDate AssetSortType = "DATE"
	AssetSortTypeSize AssetSortType = "SIZE"
	AssetSortTypeName AssetSortType = "NAME"
)

var AllAssetSortType = []AssetSortType{
	AssetSortTypeDate,
	AssetSortTypeSize,
	AssetSortTypeName,
}

func (e AssetSortType) IsValid() bool {
	switch e {
	case AssetSortTypeDate, AssetSortTypeSize, AssetSortTypeName:
		return true
	}
	return false
}

func (e AssetSortType) String() string {
	return string(e)
}

func (e *AssetSortType) UnmarshalGQL(v any) error {
	str, ok := v.(string)
	if !ok {
		return fmt.Errorf("enums must be strings")
	}

	*e = AssetSortType(str)
	if !e.IsValid() {
		return fmt.Errorf("%s is not a valid AssetSortType", str)
	}
	return nil
}

func (e AssetSortType) MarshalGQL(w io.Writer) {
	fmt.Fprint(w, strconv.Quote(e.String()))
}

func (e *AssetSortType) UnmarshalJSON(b []byte) error {
	s, err := strconv.Unquote(string(b))
	if err != nil {
		return err
	}
	return e.UnmarshalGQL(s)
}

func (e AssetSortType) MarshalJSON() ([]byte, error) {
	var buf bytes.Buffer
	e.MarshalGQL(&buf)
	return buf.Bytes(), nil
}

type EventSourceType string

const (
	EventSourceTypeTimeDriven EventSourceType = "TIME_DRIVEN"
	EventSourceTypeAPIDriven  EventSourceType = "API_DRIVEN"
)

var AllEventSourceType = []EventSourceType{
	EventSourceTypeTimeDriven,
	EventSourceTypeAPIDriven,
}

func (e EventSourceType) IsValid() bool {
	switch e {
	case EventSourceTypeTimeDriven, EventSourceTypeAPIDriven:
		return true
	}
	return false
}

func (e EventSourceType) String() string {
	return string(e)
}

func (e *EventSourceType) UnmarshalGQL(v any) error {
	str, ok := v.(string)
	if !ok {
		return fmt.Errorf("enums must be strings")
	}

	*e = EventSourceType(str)
	if !e.IsValid() {
		return fmt.Errorf("%s is not a valid EventSourceType", str)
	}
	return nil
}

func (e EventSourceType) MarshalGQL(w io.Writer) {
	fmt.Fprint(w, strconv.Quote(e.String()))
}

func (e *EventSourceType) UnmarshalJSON(b []byte) error {
	s, err := strconv.Unquote(string(b))
	if err != nil {
		return err
	}
	return e.UnmarshalGQL(s)
}

func (e EventSourceType) MarshalJSON() ([]byte, error) {
	var buf bytes.Buffer
	e.MarshalGQL(&buf)
	return buf.Bytes(), nil
}

type JobStatus string

const (
	JobStatusCancelled JobStatus = "CANCELLED"
	JobStatusCompleted JobStatus = "COMPLETED"
	JobStatusFailed    JobStatus = "FAILED"
	JobStatusPending   JobStatus = "PENDING"
	JobStatusRunning   JobStatus = "RUNNING"
)

var AllJobStatus = []JobStatus{
	JobStatusCancelled,
	JobStatusCompleted,
	JobStatusFailed,
	JobStatusPending,
	JobStatusRunning,
}

func (e JobStatus) IsValid() bool {
	switch e {
	case JobStatusCancelled, JobStatusCompleted, JobStatusFailed, JobStatusPending, JobStatusRunning:
		return true
	}
	return false
}

func (e JobStatus) String() string {
	return string(e)
}

func (e *JobStatus) UnmarshalGQL(v any) error {
	str, ok := v.(string)
	if !ok {
		return fmt.Errorf("enums must be strings")
	}

	*e = JobStatus(str)
	if !e.IsValid() {
		return fmt.Errorf("%s is not a valid JobStatus", str)
	}
	return nil
}

func (e JobStatus) MarshalGQL(w io.Writer) {
	fmt.Fprint(w, strconv.Quote(e.String()))
}

func (e *JobStatus) UnmarshalJSON(b []byte) error {
	s, err := strconv.Unquote(string(b))
	if err != nil {
		return err
	}
	return e.UnmarshalGQL(s)
}

func (e JobStatus) MarshalJSON() ([]byte, error) {
	var buf bytes.Buffer
	e.MarshalGQL(&buf)
	return buf.Bytes(), nil
}

type LogLevel string

const (
	LogLevelError LogLevel = "ERROR"
	LogLevelWarn  LogLevel = "WARN"
	LogLevelInfo  LogLevel = "INFO"
	LogLevelDebug LogLevel = "DEBUG"
	LogLevelTrace LogLevel = "TRACE"
)

var AllLogLevel = []LogLevel{
	LogLevelError,
	LogLevelWarn,
	LogLevelInfo,
	LogLevelDebug,
	LogLevelTrace,
}

func (e LogLevel) IsValid() bool {
	switch e {
	case LogLevelError, LogLevelWarn, LogLevelInfo, LogLevelDebug, LogLevelTrace:
		return true
	}
	return false
}

func (e LogLevel) String() string {
	return string(e)
}

func (e *LogLevel) UnmarshalGQL(v any) error {
	str, ok := v.(string)
	if !ok {
		return fmt.Errorf("enums must be strings")
	}

	*e = LogLevel(str)
	if !e.IsValid() {
		return fmt.Errorf("%s is not a valid LogLevel", str)
	}
	return nil
}

func (e LogLevel) MarshalGQL(w io.Writer) {
	fmt.Fprint(w, strconv.Quote(e.String()))
}

func (e *LogLevel) UnmarshalJSON(b []byte) error {
	s, err := strconv.Unquote(string(b))
	if err != nil {
		return err
	}
	return e.UnmarshalGQL(s)
}

func (e LogLevel) MarshalJSON() ([]byte, error) {
	var buf bytes.Buffer
	e.MarshalGQL(&buf)
	return buf.Bytes(), nil
}

type NodeStatus string

const (
	NodeStatusPending    NodeStatus = "PENDING"
	NodeStatusStarting   NodeStatus = "STARTING"
	NodeStatusProcessing NodeStatus = "PROCESSING"
	NodeStatusCompleted  NodeStatus = "COMPLETED"
	NodeStatusFailed     NodeStatus = "FAILED"
)

var AllNodeStatus = []NodeStatus{
	NodeStatusPending,
	NodeStatusStarting,
	NodeStatusProcessing,
	NodeStatusCompleted,
	NodeStatusFailed,
}

func (e NodeStatus) IsValid() bool {
	switch e {
	case NodeStatusPending, NodeStatusStarting, NodeStatusProcessing, NodeStatusCompleted, NodeStatusFailed:
		return true
	}
	return false
}

func (e NodeStatus) String() string {
	return string(e)
}

func (e *NodeStatus) UnmarshalGQL(v any) error {
	str, ok := v.(string)
	if !ok {
		return fmt.Errorf("enums must be strings")
	}

	*e = NodeStatus(str)
	if !e.IsValid() {
		return fmt.Errorf("%s is not a valid NodeStatus", str)
	}
	return nil
}

func (e NodeStatus) MarshalGQL(w io.Writer) {
	fmt.Fprint(w, strconv.Quote(e.String()))
}

func (e *NodeStatus) UnmarshalJSON(b []byte) error {
	s, err := strconv.Unquote(string(b))
	if err != nil {
		return err
	}
	return e.UnmarshalGQL(s)
}

func (e NodeStatus) MarshalJSON() ([]byte, error) {
	var buf bytes.Buffer
	e.MarshalGQL(&buf)
	return buf.Bytes(), nil
}

type NodeType string

const (
	NodeTypeAsset     NodeType = "ASSET"
	NodeTypeProject   NodeType = "PROJECT"
	NodeTypeUser      NodeType = "USER"
	NodeTypeWorkspace NodeType = "WORKSPACE"
)

var AllNodeType = []NodeType{
	NodeTypeAsset,
	NodeTypeProject,
	NodeTypeUser,
	NodeTypeWorkspace,
}

func (e NodeType) IsValid() bool {
	switch e {
	case NodeTypeAsset, NodeTypeProject, NodeTypeUser, NodeTypeWorkspace:
		return true
	}
	return false
}

func (e NodeType) String() string {
	return string(e)
}

func (e *NodeType) UnmarshalGQL(v any) error {
	str, ok := v.(string)
	if !ok {
		return fmt.Errorf("enums must be strings")
	}

	*e = NodeType(str)
	if !e.IsValid() {
		return fmt.Errorf("%s is not a valid NodeType", str)
	}
	return nil
}

func (e NodeType) MarshalGQL(w io.Writer) {
	fmt.Fprint(w, strconv.Quote(e.String()))
}

func (e *NodeType) UnmarshalJSON(b []byte) error {
	s, err := strconv.Unquote(string(b))
	if err != nil {
		return err
	}
	return e.UnmarshalGQL(s)
}

func (e NodeType) MarshalJSON() ([]byte, error) {
	var buf bytes.Buffer
	e.MarshalGQL(&buf)
	return buf.Bytes(), nil
}

type OrderDirection string

const (
	OrderDirectionAsc  OrderDirection = "ASC"
	OrderDirectionDesc OrderDirection = "DESC"
)

var AllOrderDirection = []OrderDirection{
	OrderDirectionAsc,
	OrderDirectionDesc,
}

func (e OrderDirection) IsValid() bool {
	switch e {
	case OrderDirectionAsc, OrderDirectionDesc:
		return true
	}
	return false
}

func (e OrderDirection) String() string {
	return string(e)
}

func (e *OrderDirection) UnmarshalGQL(v any) error {
	str, ok := v.(string)
	if !ok {
		return fmt.Errorf("enums must be strings")
	}

	*e = OrderDirection(str)
	if !e.IsValid() {
		return fmt.Errorf("%s is not a valid OrderDirection", str)
	}
	return nil
}

func (e OrderDirection) MarshalGQL(w io.Writer) {
	fmt.Fprint(w, strconv.Quote(e.String()))
}

func (e *OrderDirection) UnmarshalJSON(b []byte) error {
	s, err := strconv.Unquote(string(b))
	if err != nil {
		return err
	}
	return e.UnmarshalGQL(s)
}

func (e OrderDirection) MarshalJSON() ([]byte, error) {
	var buf bytes.Buffer
	e.MarshalGQL(&buf)
	return buf.Bytes(), nil
}

type ParameterType string

const (
	ParameterTypeText       ParameterType = "TEXT"
	ParameterTypeNumber     ParameterType = "NUMBER"
	ParameterTypeChoice     ParameterType = "CHOICE"
	ParameterTypeFileFolder ParameterType = "FILE_FOLDER"
	ParameterTypeYesNo      ParameterType = "YES_NO"
	ParameterTypeDatetime   ParameterType = "DATETIME"
	ParameterTypeColor      ParameterType = "COLOR"
)

var AllParameterType = []ParameterType{
	ParameterTypeText,
	ParameterTypeNumber,
	ParameterTypeChoice,
	ParameterTypeFileFolder,
	ParameterTypeYesNo,
	ParameterTypeDatetime,
	ParameterTypeColor,
}

func (e ParameterType) IsValid() bool {
	switch e {
	case ParameterTypeText, ParameterTypeNumber, ParameterTypeChoice, ParameterTypeFileFolder, ParameterTypeYesNo, ParameterTypeDatetime, ParameterTypeColor:
		return true
	}
	return false
}

func (e ParameterType) String() string {
	return string(e)
}

func (e *ParameterType) UnmarshalGQL(v any) error {
	str, ok := v.(string)
	if !ok {
		return fmt.Errorf("enums must be strings")
	}

	*e = ParameterType(str)
	if !e.IsValid() {
		return fmt.Errorf("%s is not a valid ParameterType", str)
	}
	return nil
}

func (e ParameterType) MarshalGQL(w io.Writer) {
	fmt.Fprint(w, strconv.Quote(e.String()))
}

func (e *ParameterType) UnmarshalJSON(b []byte) error {
	s, err := strconv.Unquote(string(b))
	if err != nil {
		return err
	}
	return e.UnmarshalGQL(s)
}

func (e ParameterType) MarshalJSON() ([]byte, error) {
	var buf bytes.Buffer
	e.MarshalGQL(&buf)
	return buf.Bytes(), nil
}

type Role string

const (
	RoleMaintainer Role = "MAINTAINER"
	RoleOwner      Role = "OWNER"
	RoleReader     Role = "READER"
	RoleWriter     Role = "WRITER"
)

var AllRole = []Role{
	RoleMaintainer,
	RoleOwner,
	RoleReader,
	RoleWriter,
}

func (e Role) IsValid() bool {
	switch e {
	case RoleMaintainer, RoleOwner, RoleReader, RoleWriter:
		return true
	}
	return false
}

func (e Role) String() string {
	return string(e)
}

func (e *Role) UnmarshalGQL(v any) error {
	str, ok := v.(string)
	if !ok {
		return fmt.Errorf("enums must be strings")
	}

	*e = Role(str)
	if !e.IsValid() {
		return fmt.Errorf("%s is not a valid Role", str)
	}
	return nil
}

func (e Role) MarshalGQL(w io.Writer) {
	fmt.Fprint(w, strconv.Quote(e.String()))
}

func (e *Role) UnmarshalJSON(b []byte) error {
	s, err := strconv.Unquote(string(b))
	if err != nil {
		return err
	}
	return e.UnmarshalGQL(s)
}

func (e Role) MarshalJSON() ([]byte, error) {
	var buf bytes.Buffer
	e.MarshalGQL(&buf)
	return buf.Bytes(), nil
}

type TimeInterval string

const (
	TimeIntervalEveryDay   TimeInterval = "EVERY_DAY"
	TimeIntervalEveryHour  TimeInterval = "EVERY_HOUR"
	TimeIntervalEveryMonth TimeInterval = "EVERY_MONTH"
	TimeIntervalEveryWeek  TimeInterval = "EVERY_WEEK"
)

var AllTimeInterval = []TimeInterval{
	TimeIntervalEveryDay,
	TimeIntervalEveryHour,
	TimeIntervalEveryMonth,
	TimeIntervalEveryWeek,
}

func (e TimeInterval) IsValid() bool {
	switch e {
	case TimeIntervalEveryDay, TimeIntervalEveryHour, TimeIntervalEveryMonth, TimeIntervalEveryWeek:
		return true
	}
	return false
}

func (e TimeInterval) String() string {
	return string(e)
}

func (e *TimeInterval) UnmarshalGQL(v any) error {
	str, ok := v.(string)
	if !ok {
		return fmt.Errorf("enums must be strings")
	}

	*e = TimeInterval(str)
	if !e.IsValid() {
		return fmt.Errorf("%s is not a valid TimeInterval", str)
	}
	return nil
}

func (e TimeInterval) MarshalGQL(w io.Writer) {
	fmt.Fprint(w, strconv.Quote(e.String()))
}

func (e *TimeInterval) UnmarshalJSON(b []byte) error {
	s, err := strconv.Unquote(string(b))
	if err != nil {
		return err
	}
	return e.UnmarshalGQL(s)
}

func (e TimeInterval) MarshalJSON() ([]byte, error) {
	var buf bytes.Buffer
	e.MarshalGQL(&buf)
	return buf.Bytes(), nil
}
