package project

import (
	"errors"
	"time"
)

var ErrInvalidAlias error = errors.New("invalid alias")

type Project struct {
	updatedAt         time.Time
	sharedToken       *string
	basicAuthPassword string
	basicAuthUsername string
	description       string
	name              string
	id                ID
	workflow          WorkflowID
	workspace         WorkspaceID
	isArchived        bool
	isBasicAuthActive bool
}

func (p *Project) BasicAuthPassword() string {
	return p.basicAuthPassword
}

func (p *Project) BasicAuthUsername() string {
	return p.basicAuthUsername
}

func (p *Project) CreatedAt() time.Time {
	return p.id.Timestamp()
}

func (p *Project) Description() string {
	return p.description
}

func (p *Project) ID() ID {
	return p.id
}

func (p *Project) IsArchived() bool {
	return p.isArchived
}

func (p *Project) IsBasicAuthActive() bool {
	return p.isBasicAuthActive
}

func (p *Project) Name() string {
	return p.name
}

func (p *Project) SharedToken() *string {
	return p.sharedToken
}

func (p *Project) SetArchived(isArchived bool) {
	p.isArchived = isArchived
	p.updatedAt = time.Now()
}

func (p *Project) SetBasicAuthPassword(basicAuthPassword string) {
	p.basicAuthPassword = basicAuthPassword
	p.updatedAt = time.Now()
}

func (p *Project) SetBasicAuthUsername(basicAuthUsername string) {
	p.basicAuthUsername = basicAuthUsername
	p.updatedAt = time.Now()
}

func (p *Project) SetIsBasicAuthActive(isBasicAuthActive bool) {
	p.isBasicAuthActive = isBasicAuthActive
	p.updatedAt = time.Now()
}

func (p *Project) SetSharedToken(sharedToken *string) {
	p.sharedToken = sharedToken
	p.updatedAt = time.Now()
}

func (p *Project) SetUpdateDescription(description string) {
	p.description = description
	p.updatedAt = time.Now()
}

func (p *Project) SetUpdateName(name string) {
	p.name = name
	p.updatedAt = time.Now()
}

func (p *Project) SetUpdateWorkspace(workspace WorkspaceID) {
	p.workspace = workspace
	p.updatedAt = time.Now()
}

func (p *Project) UpdatedAt() time.Time {
	return p.updatedAt
}

func (p *Project) Workflow() WorkflowID {
	return p.workflow
}

func (p *Project) Workspace() WorkspaceID {
	return p.workspace
}
