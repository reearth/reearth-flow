package gqldataloader

//go:generate go run github.com/vektah/dataloaden AssetLoader github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel.ID *github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel.Asset
//go:generate go run github.com/vektah/dataloaden DeploymentLoader github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel.ID *github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel.Deployment
//go:generate go run github.com/vektah/dataloaden JobLoader github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel.ID *github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel.Job
//go:generate go run github.com/vektah/dataloaden ProjectLoader github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel.ID *github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel.Project
//go:generate go run github.com/vektah/dataloaden ParameterLoader github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel.ID *github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel.Parameter
//go:generate go run github.com/vektah/dataloaden TriggerLoader github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel.ID *github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel.Trigger
//go:generate go run github.com/vektah/dataloaden UserLoader github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel.ID *github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel.User
//go:generate go run github.com/vektah/dataloaden WorkspaceLoader github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel.ID *github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel.Workspace
