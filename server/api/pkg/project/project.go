package project

import (
	"slices"
	"time"
)

type Project struct {
	updatedAt         time.Time
	sharedToken       *string
	createdBy         *UserID
	updatedBy         *UserID
	basicAuthPassword string
	basicAuthUsername string
	description       string
	name              string
	alias             string
	visibility        Visibility
	readme            string
	license           string
	topics            []string
	starredBy         []string
	id                ID
	workflow          WorkflowID
	workspace         WorkspaceID
	isArchived        bool
	isBasicAuthActive bool
	isLocked          bool
}

func (p *Project) Alias() string {
	return p.alias
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

// CreatedBy is nil for projects created before it was recorded, or by a caller
// with no user.
func (p *Project) CreatedBy() *UserID {
	return p.createdBy.CloneRef()
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

func (p *Project) IsLocked() bool {
	return p.isLocked
}

func (p *Project) IsStarredBy(uid UserID) bool {
	return slices.Contains(p.starredBy, uid.String())
}

func (p *Project) License() string {
	return p.license
}

func (p *Project) Name() string {
	return p.name
}

func (p *Project) Readme() string {
	return p.readme
}

// SetAlias validates alias before assigning it, so a rejected alias leaves the
// project unchanged.
func (p *Project) SetAlias(alias string) error {
	if err := ValidateAlias(alias, p.id); err != nil {
		return err
	}
	p.alias = alias
	p.updatedAt = time.Now()
	return nil
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

func (p *Project) SetIsLocked(isLocked bool) {
	p.isLocked = isLocked
	p.updatedAt = time.Now()
}

func (p *Project) SetLicense(license string) {
	p.license = license
	p.updatedAt = time.Now()
}

func (p *Project) SetReadme(readme string) {
	p.readme = readme
	p.updatedAt = time.Now()
}

func (p *Project) SetSharedToken(sharedToken *string) {
	p.sharedToken = sharedToken
	p.updatedAt = time.Now()
}

func (p *Project) SetTopics(topics []string) error {
	normalized, err := NormalizeTopics(topics)
	if err != nil {
		return err
	}
	p.topics = normalized
	p.updatedAt = time.Now()
	return nil
}

// SetUpdatedBy records who made the latest edit. It does not touch updatedAt:
// the setter for the edited field already did.
func (p *Project) SetUpdatedBy(uid *UserID) {
	p.updatedBy = uid.CloneRef()
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

func (p *Project) SetVisibility(visibility Visibility) error {
	if !visibility.Valid() {
		return ErrInvalidVisibility
	}
	p.visibility = visibility
	p.updatedAt = time.Now()
	return nil
}

func (p *Project) SharedToken() *string {
	return p.sharedToken
}

// Star and Unstar leave updatedAt alone: a star is not an edit, and bumping it
// would reorder lists sorted by last update.
func (p *Project) Star(uid UserID) {
	if !p.IsStarredBy(uid) {
		p.starredBy = append(slices.Clone(p.starredBy), uid.String())
	}
}

func (p *Project) StarCount() int64 {
	return int64(len(p.starredBy))
}

func (p *Project) StarredBy() []string {
	return slices.Clone(p.starredBy)
}

func (p *Project) Topics() []string {
	return slices.Clone(p.topics)
}

func (p *Project) Unstar(uid UserID) {
	p.starredBy = slices.DeleteFunc(slices.Clone(p.starredBy), func(s string) bool { return s == uid.String() })
}

func (p *Project) UpdatedAt() time.Time {
	return p.updatedAt
}

// UpdatedBy is nil until a user edits a project that predates the field.
func (p *Project) UpdatedBy() *UserID {
	return p.updatedBy.CloneRef()
}

func (p *Project) Visibility() Visibility {
	return p.visibility
}

func (p *Project) Workflow() WorkflowID {
	return p.workflow
}

func (p *Project) Workspace() WorkspaceID {
	return p.workspace
}
