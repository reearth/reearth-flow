package gql

//go:generate go run go.uber.org/mock/mockgen@latest -source=github.com/reearth/reearth-accounts/server/pkg/gqlclient/user -destination=user_mockrepo.go -package=gql -mock_names=Repo=MockUserRepo
