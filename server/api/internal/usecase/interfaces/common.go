package interfaces

import (
	"errors"
)

type ListOperation string

const (
	ListOperationAdd    ListOperation = "add"
	ListOperationMove   ListOperation = "move"
	ListOperationRemove ListOperation = "remove"
)

var (
	ErrSceneIsLocked   error = errors.New("scene is locked")
	ErrOperationDenied error = errors.New("operation denied")
	ErrFileNotIncluded error = errors.New("file not included")
	ErrFeatureNotFound error = errors.New("feature not found")
	ErrInvalidOperator error = errors.New("invalid operator")
	// ErrSnapshotNotFound: retention (KeepSnapshots) evicts snapshots, so a row
	// listed a moment ago can be gone by the time it is clicked. Distinct from a
	// server fault so the UI can say "no longer available" instead of "error".
	ErrSnapshotNotFound error = errors.New("snapshot not found")
)

type Container struct {
	Asset         Asset
	CMS           CMS
	Deployment    Deployment
	EdgeExecution EdgeExecution
	Job           Job
	Log           Log
	NodeExecution NodeExecution
	Parameter     Parameter
	Project       Project
	ProjectAccess ProjectAccess
	Trigger       Trigger
	UserFacingLog UserFacingLog
	User          User
	Workspace     Workspace
	Websocket     WebsocketClient
	WorkerConfig  WorkerConfig
}
