package workspace

type Role string

const (
	RoleReader     Role = "reader"
	RoleWriter     Role = "writer"
	RoleMaintainer Role = "maintainer"
	RoleOwner      Role = "owner"
)
