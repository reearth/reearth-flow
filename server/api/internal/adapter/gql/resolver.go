package gql

type Resolver struct{}

func NewResolver() *Resolver {
	return &Resolver{}
}

// Deployment returns DeploymentResolver implementation.
func (r *Resolver) Deployment() DeploymentResolver { return &deploymentResolver{r} }

// Job returns JobResolver implementation.
func (r *Resolver) Job() JobResolver { return &jobResolver{r} }

// Me returns MeResolver implementation.
func (r *Resolver) Me() MeResolver { return &meResolver{r} }

// Mutation returns MutationResolver implementation.
func (r *Resolver) Mutation() MutationResolver { return &mutationResolver{r} }

// Project returns ProjectResolver implementation.
func (r *Resolver) Project() ProjectResolver { return &projectResolver{r} }

// ProjectDocument returns ProjectDocumentResolver implementation.
func (r *Resolver) ProjectDocument() ProjectDocumentResolver { return &projectDocumentResolver{r} }

// Query returns QueryResolver implementation.
func (r *Resolver) Query() QueryResolver { return &queryResolver{r} }

// Subscription returns SubscriptionResolver implementation.
func (r *Resolver) Subscription() SubscriptionResolver { return &subscriptionResolver{r} }

// Trigger returns TriggerResolver implementation.
func (r *Resolver) Trigger() TriggerResolver { return &triggerResolver{r} }

// Workspace returns WorkspaceResolver implementation.
func (r *Resolver) Workspace() WorkspaceResolver { return &workspaceResolver{r} }

// WorkspaceMember returns WorkspaceMemberResolver implementation.
func (r *Resolver) WorkspaceMember() WorkspaceMemberResolver { return &workspaceMemberResolver{r} }
