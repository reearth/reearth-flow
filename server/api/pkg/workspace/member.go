package workspace

type Member interface {
	GetRole() Role
	GetMemberType() string
}

type User struct {
	Name  string
	Email string
	ID    UserID
}

type UserMember struct {
	Host   *string
	User   *User
	Role   Role
	UserID UserID
}

type IntegrationMember struct {
	InvitedBy     *User
	Role          Role
	IntegrationID IntegrationID
	InvitedByID   UserID
	Active        bool
}

func (m UserMember) GetRole() Role         { return m.Role }
func (m UserMember) GetMemberType() string { return "UserMember" }

func (m IntegrationMember) GetRole() Role         { return m.Role }
func (m IntegrationMember) GetMemberType() string { return "IntegrationMember" }
