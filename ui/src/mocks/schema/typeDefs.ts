// Comprehensive GraphQL schema for Re:Earth Flow mock server
// Based on the actual schema from server/api/gql/
export const typeDefs = `
  # Scalars
  scalar Upload
  scalar Any
  scalar DateTime
  scalar URL
  scalar Lang
  scalar FileSize
  scalar JSON

  # Core interface
  interface Node {
    id: ID!
  }

  enum NodeType {
    ASSET
    PROJECT
    USER
    WORKSPACE
  }

  # Pagination
  type PageInfo {
    totalCount: Int!
    currentPage: Int
    totalPages: Int
  }

  input PageBasedPagination {
    page: Int!
    pageSize: Int!
    orderBy: String
    orderDir: OrderDirection
  }

  enum OrderDirection {
    ASC
    DESC
  }

  input Pagination {
    page: Int
    pageSize: Int
    orderBy: String
    orderDir: OrderDirection
  }

  # User Types
  type User implements Node {
    email: String!
    host: String
    id: ID!
    name: String!
  }

  type Me {
    auths: [String!]!
    email: String!
    id: ID!
    lang: Lang!
    myWorkspace: Workspace
    myWorkspaceId: ID!
    name: String!
    workspaces: [Workspace!]!
  }

  # Workspace Types
  type Workspace implements Node {
    assets(pagination: Pagination): AssetConnection!
    id: ID!
    members: [WorkspaceMember!]!
    name: String!
    personal: Boolean!
    projects(
      includeArchived: Boolean
      pagination: Pagination
    ): ProjectConnection!
  }

  type WorkspaceMember {
    role: Role!
    user: User
    userId: ID!
  }

  enum Role {
    MAINTAINER
    OWNER
    READER
    WRITER
  }

  # Parameter Types
  type Parameter {
    createdAt: DateTime!
    id: ID!
    index: Int!
    name: String!
    projectId: ID!
    required: Boolean!
    type: ParameterType!
    updatedAt: DateTime!
    value: Any!
  }

  enum ParameterType {
    CHOICE
    COLOR
    DATETIME
    FILE_FOLDER
    MESSAGE
    NUMBER
    PASSWORD
    TEXT
    YES_NO
    ATTRIBUTE_NAME
    COORDINATE_SYSTEM
    DATABASE_CONNECTION
    GEOMETRY
    REPROJECTION_FILE
    WEB_CONNECTION
  }

  # Project Types
  type Project implements Node {
    basicAuthPassword: String!
    basicAuthUsername: String!
    createdAt: DateTime!
    description: String!
    deployment: Deployment
    id: ID!
    isArchived: Boolean!
    isBasicAuthActive: Boolean!
    name: String!
    parameters: [Parameter!]!
    updatedAt: DateTime!
    sharedToken: String
    version: Int!
    workspace: Workspace
    workspaceId: ID!
  }

  # Asset Types

  enum ArchiveExtractionStatus {
    SKIPPED
    PENDING
    IN_PROGRESS
    DONE
    FAILED
  }

  type Asset implements Node {
    id: ID!
    workspaceId: ID!
    createdAt: DateTime!
    fileName: String!
    size: FileSize!
    contentType: String!
    name: String!
    url: String!
    uuid: String!
    flatFiles: Boolean!
    public: Boolean!
    archiveExtractionStatus: ArchiveExtractionStatus
    Workspace: Workspace
  }

  enum AssetSortType {
    DATE
    NAME
    SIZE
  }

  # Deployment Types
  type Deployment implements Node {
    createdAt: DateTime!
    description: String!
    headId: ID
    isHead: Boolean!
    id: ID!
    project: Project
    projectId: ID
    updatedAt: DateTime!
    version: String!
    workflowUrl: String!
    workspace: Workspace
    workspaceId: ID!
  }

  # Job Types
  type Job implements Node {
    completedAt: DateTime
    deployment: Deployment
    deploymentId: ID!
    debug: Boolean
    id: ID!
    logsURL: String
    workerLogsURL: String
    outputURLs: [String!]
    startedAt: DateTime!
    status: JobStatus!
    workspace: Workspace
    workspaceId: ID!
    logs(since: DateTime!): [Log]
  }

  enum JobStatus {
    CANCELLED
    COMPLETED
    FAILED
    PENDING
    RUNNING
  }

  # Log Types
  enum LogLevel {
    ERROR
    WARN
    INFO
    DEBUG
    TRACE
  }

  type Log {
    jobId: ID!
    nodeId: ID
    timestamp: DateTime!
    logLevel: LogLevel!
    message: String!
  }

  # Node Execution Types
  type NodeExecution {
    id: ID!
    nodeId: ID!
    jobId: ID!
    status: NodeExecutionStatus!
    startedAt: DateTime
    completedAt: DateTime
    logs: [Log!]!
  }

  enum NodeExecutionStatus {
    PENDING
    RUNNING
    COMPLETED
    FAILED
    CANCELLED
  }

  # Document Types
  type ProjectDocument {
    id: ID!
    projectId: ID!
    content: JSON!
    createdAt: DateTime!
    updatedAt: DateTime!
  }

  type ProjectSnapshot {
    id: ID!
    projectId: ID!
    content: JSON!
    createdAt: DateTime!
    metadata: ProjectSnapshotMetadata!
  }

  type ProjectSnapshotMetadata {
    id: ID!
    version: String!
    description: String
    createdAt: DateTime!
  }

  # Trigger Types
  type Trigger implements Node {
    id: ID!
    workspaceId: ID!
    projectId: ID!
    name: String!
    enabled: Boolean!
    schedule: String
    webhook: String
    createdAt: DateTime!
    updatedAt: DateTime!
  }

  # Sharing Types
  type ProjectSharingInfoPayload {
    projectId: ID!
    enabled: Boolean!
    token: String
  }

  type SharedProjectPayload {
    project: Project!
    sharedToken: String!
  }

  # CMS Types

  type CMSProject {
    id: ID!
    name: String!
    alias: String!
    description: String
    license: String
    readme: String
    workspaceId: ID!
    visibility: CMSVisibility!
    createdAt: DateTime!
    updatedAt: DateTime!
  }

  enum CMSVisibility {
    PUBLIC
    PRIVATE
  }

  type CMSModel {
    id: ID!
    projectId: ID!
    name: String!
    description: String!
    key: String!
    schema: CMSSchema!
    publicApiEp: String!
    editorUrl: String!
    createdAt: DateTime!
    updatedAt: DateTime!
  }
  type CMSSchema {
    schemaId: ID!
    fields: [CMSSchemaField!]!
  }

  type CMSSchemaField {
    fieldId: ID!
    name: String!
    type: CMSSchemaFieldType!
    key: String!
    description: String!
  }

  enum CMSSchemaFieldType {
    TEXT
    TEXTAREA
    RICHTEXT
    MARKDOWNTEXT
    ASSET
    DATE
    BOOL
    SELECT
    TAG
    INTEGER
    NUMBER
    REFERENCE
    CHECKBOX
    URL
    GROUP
    GEOMETRYOBJECT
    GEOMETRYEDITOR
  }

  type CMSItem {
    id: ID!
    fields: JSON!
    createdAt: DateTime!
    updatedAt: DateTime!
  }

  # Connection Types
  type AssetConnection {
    nodes: [Asset]!
    pageInfo: PageInfo!
    totalCount: Int!
  }

  type ProjectConnection {
    nodes: [Project]!
    pageInfo: PageInfo!
    totalCount: Int!
  }

  type JobConnection {
    nodes: [Job]!
    pageInfo: PageInfo!
    totalCount: Int!
  }

  type DeploymentConnection {
    nodes: [Deployment]!
    pageInfo: PageInfo!
    totalCount: Int!
  }

  type TriggerConnection {
    nodes: [Trigger]!
    pageInfo: PageInfo!
    totalCount: Int!
  }

  type CMSItemsConnection {
    items: [CMSItem!]!
    totalCount: Int!
  }

  # Input Types - User
  input SignupInput {
    userId: ID
    lang: Lang
    workspaceId: ID
    secret: String
  }

  input UpdateMeInput {
    name: String
    email: String
    password: String
    passwordConfirmation: String
    lang: Lang
  }

  input RemoveMyAuthInput {
    auth: String!
  }

  input DeleteMeInput {
    userId: ID!
  }

  # Input Types - Workspace
  input CreateWorkspaceInput {
    name: String!
  }

  input UpdateWorkspaceInput {
    workspaceId: ID!
    name: String!
  }

  input AddMemberToWorkspaceInput {
    workspaceId: ID!
    userId: ID!
    role: Role!
  }

  input RemoveMemberFromWorkspaceInput {
    workspaceId: ID!
    userId: ID!
  }

  input UpdateMemberOfWorkspaceInput {
    workspaceId: ID!
    userId: ID!
    role: Role!
  }

  input DeleteWorkspaceInput {
    workspaceId: ID!
  }

  # Input Types - Project
  input CreateProjectInput {
    workspaceId: ID!
    name: String
    description: String
    archived: Boolean
  }

  input UpdateProjectInput {
    projectId: ID!
    name: String
    description: String
    archived: Boolean
    isBasicAuthActive: Boolean
    basicAuthUsername: String
    basicAuthPassword: String
  }

  input DeleteProjectInput {
    projectId: ID!
  }

  input RunProjectInput {
    projectId: ID!
    workspaceId: ID!
    file: Upload!
  }

  # Input Types - Parameter
  input DeclareParameterInput {
    name: String!
    type: ParameterType!
    required: Boolean!
    value: Any
    index: Int
  }

  input UpdateParameterValueInput {
    value: Any!
  }

  input UpdateParameterOrderInput {
    paramId: ID!
    newIndex: Int!
  }

  input RemoveParameterInput {
    paramId: ID!
  }

  # Input Types - Deployment
  input CreateDeploymentInput {
    workspaceId: ID!
    file: Upload!
    projectId: ID
    description: String!
  }

  input DeleteDeploymentInput {
    deploymentId: ID!
  }

  input ExecuteDeploymentInput {
    deploymentId: ID!
  }

  input GetHeadInput {
    workspaceId: ID!
    projectId: ID
  }

  input GetByVersionInput {
    workspaceId: ID!
    projectId: ID
    version: String!
  }

  input UpdateDeploymentInput {
    deploymentId: ID!
    file: Upload
    description: String
  }

  # Input Types - Job
  input CancelJobInput {
    jobId: ID!
  }

  # Input Types - Asset
  input CreateAssetInput {
    workspaceId: ID!
    file: Upload!
    name: String
  }

  input UpdateAssetInput {
    assetId: ID!
    name: String
  }

  input DeleteAssetInput {
    assetId: ID!
  }

  # Payload Types - User
  type UpdateMePayload {
    me: Me!
  }

  type SignupPayload {
    user: User!
    workspace: Workspace!
  }

  type DeleteMePayload {
    userId: ID!
  }

  # Payload Types - Workspace
  type CreateWorkspacePayload {
    workspace: Workspace!
  }

  type UpdateWorkspacePayload {
    workspace: Workspace!
  }

  type AddMemberToWorkspacePayload {
    workspace: Workspace!
  }

  type RemoveMemberFromWorkspacePayload {
    workspace: Workspace!
  }

  type UpdateMemberOfWorkspacePayload {
    workspace: Workspace!
  }

  type DeleteWorkspacePayload {
    workspaceId: ID!
  }

  # Payload Types - Project
  type ProjectPayload {
    project: Project!
  }

  type DeleteProjectPayload {
    projectId: ID!
  }

  type RunProjectPayload {
    job: Job!
  }

  # Payload Types - Deployment
  type DeploymentPayload {
    deployment: Deployment!
  }

  type DeleteDeploymentPayload {
    deploymentId: ID!
  }

  type JobPayload {
    job: Job!
  }

  # Payload Types - Job
  type CancelJobPayload {
    job: Job
  }

  # Payload Types - Asset

  type CreateAssetPayload {
    asset: Asset!
  }

  type UpdateAssetPayload {
    asset: Asset!
  }

  type DeleteAssetPayload {
    assetId: ID!
  }

  # Root Types
  type Query {
    # Core queries
    node(id: ID!, type: NodeType!): Node
    nodes(id: [ID!]!, type: NodeType!): [Node]!

    # User queries
    me: Me
    searchUser(nameOrEmail: String!): User

    # Workspace queries - implicit through Me.workspaces

    # Project queries
    projects(
      workspaceId: ID!
      includeArchived: Boolean
      pagination: PageBasedPagination!
    ): ProjectConnection!
    projectSharingInfo(projectId: ID!): ProjectSharingInfoPayload!
    sharedProject(token: String!): SharedProjectPayload!

    # Asset queries
    assets(
      workspaceId: ID!
      pagination: PageBasedPagination!
      keyword: String
      sort: AssetSortType
    ): AssetConnection!

    # Deployment queries
    deployments(workspaceId: ID!, pagination: PageBasedPagination!): DeploymentConnection!
    deploymentByVersion(input: GetByVersionInput!): Deployment
    deploymentHead(input: GetHeadInput!): Deployment
    deploymentVersions(workspaceId: ID!, projectId: ID): [Deployment!]!

    # Job queries
    jobs(workspaceId: ID!, pagination: PageBasedPagination!): JobConnection!
    job(id: ID!): Job

    # Node execution queries
    nodeExecution(jobId: ID!, nodeId: ID!): NodeExecution

    # Document queries
    latestProjectSnapshot(projectId: ID!): ProjectDocument
    projectSnapshot(projectId: ID!, version: String!): ProjectSnapshot!
    projectHistory(projectId: ID!, pagination: PageBasedPagination!): [ProjectSnapshotMetadata!]!

    # Trigger queries
    triggers(workspaceId: ID!, pagination: PageBasedPagination!): TriggerConnection!

    # CMS queries
    cmsProject(projectIdOrAlias: ID!): CMSProject
    cmsProjects(workspaceId: ID!, publicOnly: Boolean): [CMSProject!]!
    cmsModels(projectId: ID!): [CMSModel!]!
    cmsItems(projectId: ID!, modelId: ID!, page: Int, pageSize: Int): CMSItemsConnection!
    cmsModelExportUrl(projectId: ID!, modelId: ID!): String!
  }

  type Mutation {
    # User mutations
    signup(input: SignupInput!): SignupPayload
    updateMe(input: UpdateMeInput!): UpdateMePayload
    removeMyAuth(input: RemoveMyAuthInput!): UpdateMePayload
    deleteMe(input: DeleteMeInput!): DeleteMePayload

    # Workspace mutations
    createWorkspace(input: CreateWorkspaceInput!): CreateWorkspacePayload
    deleteWorkspace(input: DeleteWorkspaceInput!): DeleteWorkspacePayload
    updateWorkspace(input: UpdateWorkspaceInput!): UpdateWorkspacePayload
    addMemberToWorkspace(input: AddMemberToWorkspaceInput!): AddMemberToWorkspacePayload
    removeMemberFromWorkspace(input: RemoveMemberFromWorkspaceInput!): RemoveMemberFromWorkspacePayload
    updateMemberOfWorkspace(input: UpdateMemberOfWorkspaceInput!): UpdateMemberOfWorkspacePayload

    # Project mutations
    createProject(input: CreateProjectInput!): ProjectPayload
    updateProject(input: UpdateProjectInput!): ProjectPayload
    deleteProject(input: DeleteProjectInput!): DeleteProjectPayload
    runProject(input: RunProjectInput!): RunProjectPayload

    # Parameter mutations
    declareParameter(projectId: ID!, input: DeclareParameterInput!): Parameter!
    updateParameterValue(paramId: ID!, input: UpdateParameterValueInput!): Parameter!
    updateParameterOrder(projectId: ID!, input: UpdateParameterOrderInput!): [Parameter!]!
    removeParameter(input: RemoveParameterInput!): Boolean!

    # Asset mutations
    createAsset(input: CreateAssetInput!): CreateAssetPayload
    updateAsset(input: UpdateAssetInput!): UpdateAssetPayload
    deleteAsset(input: DeleteAssetInput!): DeleteAssetPayload

    # Deployment mutations
    createDeployment(input: CreateDeploymentInput!): DeploymentPayload
    updateDeployment(input: UpdateDeploymentInput!): DeploymentPayload
    deleteDeployment(input: DeleteDeploymentInput!): DeleteDeploymentPayload
    executeDeployment(input: ExecuteDeploymentInput!): JobPayload

    # Job mutations
    cancelJob(input: CancelJobInput!): CancelJobPayload!
  }

  type Subscription {
    # Job subscriptions
    jobStatus(jobId: ID!): JobStatus!

    # Log subscriptions
    logs(jobId: ID!): Log
  }

  schema {
    query: Query
    mutation: Mutation
    subscription: Subscription
  }
`;
