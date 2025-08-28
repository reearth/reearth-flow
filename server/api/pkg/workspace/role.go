package workspace

type Role string

const (
	RoleReader     Role = "READER"
	RoleWriter     Role = "WRITER"
	RoleMaintainer Role = "MAINTAINER"
	RoleOwner      Role = "OWNER"
)
