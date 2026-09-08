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
	ErrSceneIsLocked    error = errors.New("scene is locked")
	ErrOperationDenied  error = errors.New("operation denied")
	ErrFileNotIncluded  error = errors.New("file not included")
	ErrFeatureNotFound  error = errors.New("feature not found")
	ErrInvalidOperator  error = errors.New("invalid operator")
	ErrSnapshotNotFound error = errors.New("snapshot not found")
)

type Container struct {
	Asset           Asset
	CMS             CMS
	Deployment      Deployment
	Job             Job
	Log             Log
	NodeDiagnostics NodeDiagnostics
	Parameter       Parameter
	Project         Project
	ProjectAccess   ProjectAccess
	Trigger         Trigger
	UserFacingLog   UserFacingLog
	User            User
	Workspace       Workspace
	Websocket       WebsocketClient
	WorkerConfig    WorkerConfig
}
