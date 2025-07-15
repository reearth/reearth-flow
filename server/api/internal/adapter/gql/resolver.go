package gql

// THIS CODE WILL BE UPDATED WITH SCHEMA CHANGES. PREVIOUS IMPLEMENTATION FOR SCHEMA CHANGES WILL BE KEPT IN THE COMMENT SECTION. IMPLEMENTATION FOR UNCHANGED SCHEMA WILL BE KEPT.

type Resolver struct{}

func NewResolver() ResolverRoot {
	return &Resolver{}
}

func (r *Resolver) Deployment() DeploymentResolver {
	return &deploymentResolver{r}
}

func (r *Resolver) Job() JobResolver {
	return &jobResolver{r}
}

func (r *Resolver) Me() MeResolver {
	return &meResolver{r}
}

func (r *Resolver) Mutation() MutationResolver {
	return &mutationResolver{r}
}

func (r *Resolver) Project() ProjectResolver {
	return &projectResolver{r}
}

func (r *Resolver) ProjectDocument() ProjectDocumentResolver {
	return &projectDocumentResolver{r}
}

func (r *Resolver) Query() QueryResolver {
	return &queryResolver{r}
}

func (r *Resolver) Subscription() SubscriptionResolver {
	return &subscriptionResolver{r}
}

func (r *Resolver) Trigger() TriggerResolver {
	return &triggerResolver{r}
}

func (r *Resolver) Workspace() WorkspaceResolver {
	return &workspaceResolver{r}
}

func (r *Resolver) WorkspaceMember() WorkspaceMemberResolver {
	return &workspaceMemberResolver{r}
}
