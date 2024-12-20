// Code generated by github.com/99designs/gqlgen, DO NOT EDIT.

package gqlmodel

import (
	"fmt"
	"io"
	"strconv"
	"time"

	"github.com/99designs/gqlgen/graphql"
	"github.com/reearth/reearthx/usecasex"
	"golang.org/x/text/language"
)

type Node interface {
	IsNode()
	GetID() ID
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
	ContentType string     `json:"contentType"`
	CreatedAt   time.Time  `json:"createdAt"`
	ID          ID         `json:"id"`
	Name        string     `json:"name"`
	Size        int64      `json:"size"`
	URL         string     `json:"url"`
	WorkspaceID ID         `json:"workspaceId"`
	Workspace   *Workspace `json:"Workspace,omitempty"`
}

func (Asset) IsNode()        {}
func (this Asset) GetID() ID { return this.ID }

type AssetConnection struct {
	Edges      []*AssetEdge `json:"edges"`
	Nodes      []*Asset     `json:"nodes"`
	PageInfo   *PageInfo    `json:"pageInfo"`
	TotalCount int          `json:"totalCount"`
}

type AssetEdge struct {
	Cursor usecasex.Cursor `json:"cursor"`
	Node   *Asset          `json:"node,omitempty"`
}

type CreateAssetInput struct {
	WorkspaceID ID             `json:"workspaceId"`
	File        graphql.Upload `json:"file"`
}

type CreateAssetPayload struct {
	Asset *Asset `json:"asset"`
}

type CreateDeploymentInput struct {
	WorkspaceID ID             `json:"workspaceId"`
	ProjectID   ID             `json:"projectId"`
	File        graphql.Upload `json:"file"`
	Description *string        `json:"description,omitempty"`
}

type CreateProjectInput struct {
	WorkspaceID ID      `json:"workspaceId"`
	Name        *string `json:"name,omitempty"`
	Description *string `json:"description,omitempty"`
	Archived    *bool   `json:"archived,omitempty"`
}

type CreateWorkspaceInput struct {
	Name string `json:"name"`
}

type CreateWorkspacePayload struct {
	Workspace *Workspace `json:"workspace"`
}

type DeclareParameterInput struct {
	Name     string        `json:"name"`
	Type     ParameterType `json:"type"`
	Required bool          `json:"required"`
	Value    interface{}   `json:"value,omitempty"`
	Index    *int          `json:"index,omitempty"`
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
	ID          ID         `json:"id"`
	Project     *Project   `json:"project,omitempty"`
	ProjectID   ID         `json:"projectId"`
	UpdatedAt   time.Time  `json:"updatedAt"`
	Version     string     `json:"version"`
	WorkflowURL string     `json:"workflowUrl"`
	Workspace   *Workspace `json:"workspace,omitempty"`
	WorkspaceID ID         `json:"workspaceId"`
}

func (Deployment) IsNode()        {}
func (this Deployment) GetID() ID { return this.ID }

type DeploymentConnection struct {
	Edges      []*DeploymentEdge `json:"edges"`
	Nodes      []*Deployment     `json:"nodes"`
	PageInfo   *PageInfo         `json:"pageInfo"`
	TotalCount int               `json:"totalCount"`
}

type DeploymentEdge struct {
	Cursor usecasex.Cursor `json:"cursor"`
	Node   *Deployment     `json:"node,omitempty"`
}

type DeploymentPayload struct {
	Deployment *Deployment `json:"deployment"`
}

type ExecuteDeploymentInput struct {
	DeploymentID ID `json:"deploymentId"`
}

type Job struct {
	CompletedAt  *time.Time  `json:"completedAt,omitempty"`
	Deployment   *Deployment `json:"deployment,omitempty"`
	DeploymentID ID          `json:"deploymentId"`
	ID           ID          `json:"id"`
	StartedAt    time.Time   `json:"startedAt"`
	Status       JobStatus   `json:"status"`
	Workspace    *Workspace  `json:"workspace,omitempty"`
	WorkspaceID  ID          `json:"workspaceId"`
}

func (Job) IsNode()        {}
func (this Job) GetID() ID { return this.ID }

type JobConnection struct {
	Edges      []*JobEdge `json:"edges"`
	Nodes      []*Job     `json:"nodes"`
	PageInfo   *PageInfo  `json:"pageInfo"`
	TotalCount int        `json:"totalCount"`
}

type JobEdge struct {
	Cursor usecasex.Cursor `json:"cursor"`
	Node   *Job            `json:"node,omitempty"`
}

type JobPayload struct {
	Job *Job `json:"job"`
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

type PageInfo struct {
	EndCursor       *usecasex.Cursor `json:"endCursor,omitempty"`
	HasNextPage     bool             `json:"hasNextPage"`
	HasPreviousPage bool             `json:"hasPreviousPage"`
	StartCursor     *usecasex.Cursor `json:"startCursor,omitempty"`
}

type Pagination struct {
	First  *int             `json:"first,omitempty"`
	Last   *int             `json:"last,omitempty"`
	After  *usecasex.Cursor `json:"after,omitempty"`
	Before *usecasex.Cursor `json:"before,omitempty"`
}

type Parameter struct {
	CreatedAt time.Time     `json:"createdAt"`
	ID        ID            `json:"id"`
	Index     int           `json:"index"`
	Name      string        `json:"name"`
	ProjectID ID            `json:"projectId"`
	Required  bool          `json:"required"`
	Type      ParameterType `json:"type"`
	UpdatedAt time.Time     `json:"updatedAt"`
	Value     interface{}   `json:"value"`
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
	Version           int          `json:"version"`
	Workspace         *Workspace   `json:"workspace,omitempty"`
	WorkspaceID       ID           `json:"workspaceId"`
}

func (Project) IsNode()        {}
func (this Project) GetID() ID { return this.ID }

type ProjectConnection struct {
	Edges      []*ProjectEdge `json:"edges"`
	Nodes      []*Project     `json:"nodes"`
	PageInfo   *PageInfo      `json:"pageInfo"`
	TotalCount int            `json:"totalCount"`
}

type ProjectEdge struct {
	Cursor usecasex.Cursor `json:"cursor"`
	Node   *Project        `json:"node,omitempty"`
}

type ProjectPayload struct {
	Project *Project `json:"project"`
}

type Query struct {
}

type RemoveAssetInput struct {
	AssetID ID `json:"assetId"`
}

type RemoveAssetPayload struct {
	AssetID ID `json:"assetId"`
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

type RunProjectInput struct {
	ProjectID   ID             `json:"projectId"`
	WorkspaceID ID             `json:"workspaceId"`
	File        graphql.Upload `json:"file"`
}

type RunProjectPayload struct {
	ProjectID ID   `json:"projectId"`
	Started   bool `json:"started"`
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
	JobStatus JobStatus `json:"jobStatus"`
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

type UpdateParameterOrderInput struct {
	ParamID  ID  `json:"paramId"`
	NewIndex int `json:"newIndex"`
}

type UpdateParameterValueInput struct {
	Value interface{} `json:"value"`
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

func (e *AssetSortType) UnmarshalGQL(v interface{}) error {
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

type JobStatus string

const (
	JobStatusPending   JobStatus = "PENDING"
	JobStatusRunning   JobStatus = "RUNNING"
	JobStatusCompleted JobStatus = "COMPLETED"
	JobStatusFailed    JobStatus = "FAILED"
)

var AllJobStatus = []JobStatus{
	JobStatusPending,
	JobStatusRunning,
	JobStatusCompleted,
	JobStatusFailed,
}

func (e JobStatus) IsValid() bool {
	switch e {
	case JobStatusPending, JobStatusRunning, JobStatusCompleted, JobStatusFailed:
		return true
	}
	return false
}

func (e JobStatus) String() string {
	return string(e)
}

func (e *JobStatus) UnmarshalGQL(v interface{}) error {
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

func (e *NodeType) UnmarshalGQL(v interface{}) error {
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

type ParameterType string

const (
	ParameterTypeChoice             ParameterType = "CHOICE"
	ParameterTypeColor              ParameterType = "COLOR"
	ParameterTypeDatetime           ParameterType = "DATETIME"
	ParameterTypeFileFolder         ParameterType = "FILE_FOLDER"
	ParameterTypeMessage            ParameterType = "MESSAGE"
	ParameterTypeNumber             ParameterType = "NUMBER"
	ParameterTypePassword           ParameterType = "PASSWORD"
	ParameterTypeText               ParameterType = "TEXT"
	ParameterTypeYesNo              ParameterType = "YES_NO"
	ParameterTypeAttributeName      ParameterType = "ATTRIBUTE_NAME"
	ParameterTypeCoordinateSystem   ParameterType = "COORDINATE_SYSTEM"
	ParameterTypeDatabaseConnection ParameterType = "DATABASE_CONNECTION"
	ParameterTypeGeometry           ParameterType = "GEOMETRY"
	ParameterTypeReprojectionFile   ParameterType = "REPROJECTION_FILE"
	ParameterTypeWebConnection      ParameterType = "WEB_CONNECTION"
)

var AllParameterType = []ParameterType{
	ParameterTypeChoice,
	ParameterTypeColor,
	ParameterTypeDatetime,
	ParameterTypeFileFolder,
	ParameterTypeMessage,
	ParameterTypeNumber,
	ParameterTypePassword,
	ParameterTypeText,
	ParameterTypeYesNo,
	ParameterTypeAttributeName,
	ParameterTypeCoordinateSystem,
	ParameterTypeDatabaseConnection,
	ParameterTypeGeometry,
	ParameterTypeReprojectionFile,
	ParameterTypeWebConnection,
}

func (e ParameterType) IsValid() bool {
	switch e {
	case ParameterTypeChoice, ParameterTypeColor, ParameterTypeDatetime, ParameterTypeFileFolder, ParameterTypeMessage, ParameterTypeNumber, ParameterTypePassword, ParameterTypeText, ParameterTypeYesNo, ParameterTypeAttributeName, ParameterTypeCoordinateSystem, ParameterTypeDatabaseConnection, ParameterTypeGeometry, ParameterTypeReprojectionFile, ParameterTypeWebConnection:
		return true
	}
	return false
}

func (e ParameterType) String() string {
	return string(e)
}

func (e *ParameterType) UnmarshalGQL(v interface{}) error {
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

func (e *Role) UnmarshalGQL(v interface{}) error {
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
