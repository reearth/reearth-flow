package workspace

type Member interface {
	GetRole() Role
	GetMemberType() string
}

type UserID = string
type IntegrationID = string

type User struct {
	ID    UserID
	Name  string
	Email string
}

type UserMember struct {
	UserID UserID
	Role   Role
	Host   *string
	User   *User
}

type IntegrationMember struct {
	IntegrationID IntegrationID
	Role          Role
	Active        bool
	InvitedByID   UserID
	InvitedBy     *User
}

func (m UserMember) GetRole() Role         { return m.Role }
func (m UserMember) GetMemberType() string { return "UserMember" }

func (m IntegrationMember) GetRole() Role         { return m.Role }
func (m IntegrationMember) GetMemberType() string { return "IntegrationMember" }
