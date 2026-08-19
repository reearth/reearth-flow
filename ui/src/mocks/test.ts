import { makeExecutableSchema } from "@graphql-tools/schema";
import { graphql } from "graphql";

import { resolvers } from "./schema/resolvers";
import { typeDefs } from "./schema/typeDefs";

// Create executable schema for testing
const schema = makeExecutableSchema({
  typeDefs,
  resolvers,
});

// Test the mock server components directly
export const testMockServer = async () => {
  console.log("🧪 Testing Re:Earth Flow Mock Server Components\n");

  try {
    // Test 1: Schema validation
    console.log("📋 Test 1: Schema Validation");
    if (schema) {
      console.log("✅ Schema created successfully");
      const typeMap = schema.getTypeMap();
      console.log(`📊 Schema contains ${Object.keys(typeMap).length} types`);

      // Check essential types
      const essentialTypes = [
        "Query",
        "Mutation",
        "Subscription",
        "Me",
        "User",
        "Workspace",
        "Project",
        "Job",
      ];
      for (const type of essentialTypes) {
        if (typeMap[type]) {
          console.log(`✅ Type '${type}' exists`);
        } else {
          console.log(`❌ Type '${type}' missing`);
        }
      }
    }

    // Collected so the summary below can report a real verdict.
    const failures: string[] = [];

    // Test 2: Query execution - Me
    console.log("\n📋 Test 2: Me Query Execution");
    const meQuery = `
      query Me {
        me {
          id
          name
          email
          lang
          auths
          myWorkspaceId
          myWorkspace {
            id
            name
            personal
          }
        }
      }
    `;

    const meResult = await graphql({
      schema,
      source: meQuery,
    });

    if (meResult.errors) {
      console.error("❌ Me query failed:", meResult.errors);
      failures.push("me");
    } else {
      console.log("✅ Me query successful");
      console.log(
        "📊 Current user:",
        JSON.stringify(meResult.data?.me, null, 2),
      );
    }

    // Test 3: Projects query with pagination
    console.log("\n📋 Test 3: Projects Query with Pagination");
    const projectsQuery = `
      query Projects($workspaceId: ID!, $pagination: PageBasedPagination!) {
        projects(workspaceId: $workspaceId, pagination: $pagination) {
          nodes {
            id
            name
            description
            isArchived
            parameters {
              id
              name
              type
              value
            }
          }
          pageInfo {
            totalCount
            currentPage
            totalPages
          }
        }
      }
    `;

    const projectsResult = await graphql({
      schema,
      source: projectsQuery,
      variableValues: {
        workspaceId: "workspace-1",
        pagination: { page: 1, pageSize: 5 },
      },
    });

    if (projectsResult.errors) {
      console.error("❌ Projects query failed:", projectsResult.errors);
      failures.push("projects");
    } else {
      console.log("✅ Projects query successful");
      const projects = (projectsResult.data as any)?.projects;
      console.log(`📊 Found ${projects?.pageInfo?.totalCount} projects`);
      console.log(
        "📄 First project:",
        JSON.stringify(projects?.nodes?.[0], null, 2),
      );
    }

    // Test 4: Jobs query
    console.log("\n📋 Test 4: Jobs Query");
    const jobsQuery = `
      query Jobs($workspaceId: ID!, $pagination: PageBasedPagination!) {
        jobs(workspaceId: $workspaceId, pagination: $pagination) {
          nodes {
            id
            status
            debug
            startedAt
            completedAt
            deployment {
              id
              version
              description
            }
            workspace {
              id
              name
            }
          }
          pageInfo {
            totalCount
          }
        }
      }
    `;

    const jobsResult = await graphql({
      schema,
      source: jobsQuery,
      variableValues: {
        workspaceId: "workspace-1",
        pagination: { page: 1, pageSize: 10 },
      },
    });

    if (jobsResult.errors) {
      console.error("❌ Jobs query failed:", jobsResult.errors);
      failures.push("jobs");
    } else {
      console.log("✅ Jobs query successful");
      const jobs = (jobsResult.data as any)?.jobs;
      console.log(`📊 Found ${jobs?.pageInfo?.totalCount} jobs`);
      console.log(
        "⚡ Job statuses:",
        jobs?.nodes?.map((job: any) => `${job.id}: ${job.status}`),
      );
    }

    // Test 5: Create project mutation
    console.log("\n📋 Test 5: Create Project Mutation");
    const createProjectMutation = `
      mutation CreateProject($input: CreateProjectInput!) {
        createProject(input: $input) {
          project {
            id
            name
            description
            workspaceId
            isArchived
            version
            createdAt
          }
        }
      }
    `;

    const createProjectResult = await graphql({
      schema,
      source: createProjectMutation,
      variableValues: {
        input: {
          name: "Test Project from Mock",
          description: "Created during mock server testing",
          workspaceId: "workspace-1",
        },
      },
    });

    if (createProjectResult.errors) {
      console.error(
        "❌ Create project mutation failed:",
        createProjectResult.errors,
      );
      failures.push("createProject");
    } else {
      console.log("✅ Create project mutation successful");
      const project = (createProjectResult.data as any)?.createProject?.project;
      console.log("🆕 Created project:", JSON.stringify(project, null, 2));
    }

    // Test 6: Run project mutation
    console.log("\n📋 Test 6: Run Project Mutation");
    const runProjectMutation = `
      mutation RunProject($input: RunProjectInput!) {
        runProject(input: $input) {
          job {
            id
            status
            deploymentId
            workspaceId
            startedAt
            debug
          }
        }
      }
    `;

    const runProjectResult = await graphql({
      schema,
      source: runProjectMutation,
      variableValues: {
        input: {
          projectId: "project-1",
          workspaceId: "workspace-1",
          file: new Blob([JSON.stringify({ name: "Workflow 1" })], {
            type: "application/json",
          }),
        },
      },
    });

    if (runProjectResult.errors) {
      console.error("❌ Run project mutation failed:", runProjectResult.errors);
      failures.push("runProject");
    } else {
      console.log("✅ Run project mutation successful");
      const job = (runProjectResult.data as any)?.runProject?.job;
      console.log("🚀 Created job:", JSON.stringify(job, null, 2));
    }

    // Test 7: Node interface resolution
    console.log("\n📋 Test 7: Node Interface Resolution");
    const nodeQuery = `
      query Node($id: ID!, $type: NodeType!) {
        node(id: $id, type: $type) {
          id
          ... on User {
            name
            email
          }
          ... on Workspace {
            name
            personal
          }
          ... on Project {
            name
            description
          }
        }
      }
    `;

    const nodeResult = await graphql({
      schema,
      source: nodeQuery,
      variableValues: {
        id: "user-1",
        type: "USER",
      },
    });

    if (nodeResult.errors) {
      console.error("❌ Node query failed:", nodeResult.errors);
      failures.push("node");
    } else {
      console.log("✅ Node interface resolution successful");
      console.log(
        "🔗 Node result:",
        JSON.stringify(nodeResult.data?.node, null, 2),
      );
    }

    if (failures.length) {
      // Previously this always printed success, so real failures were logged and
      // then contradicted two lines later, training everyone to ignore them.
      console.error(
        `\n❌ ${failures.length} mock server test(s) FAILED: ${failures.join(", ")}`,
      );
      return;
    }

    console.log("\n🎉 All mock server component tests completed successfully!");
    console.log("✨ Mock server is ready for use with Re:Earth Flow UI");

    return true;
  } catch (error) {
    console.error("💥 Mock server test failed:", error);
    return false;
  }
};

// Browser test for live GraphQL endpoint
export const testLiveEndpoint = async () => {
  console.log("🌐 Testing Live GraphQL Endpoint...\n");

  try {
    const meQuery = `
      query Me {
        me {
          id
          name
          email
          lang
          myWorkspaceId
        }
      }
    `;

    const response = await fetch("/api/graphql", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer mock-token",
      },
      body: JSON.stringify({
        query: meQuery,
      }),
    });

    const result = await response.json();
    console.log("✅ Live endpoint test successful:", result);
    return true;
  } catch (error) {
    console.error("❌ Live endpoint test failed:", error);
    return false;
  }
};

// Auto-run tests
if (typeof window !== "undefined" && import.meta.env?.DEV) {
  // Make test functions available globally for manual testing
  (window as any).testMockServer = testMockServer;
  (window as any).testLiveEndpoint = testLiveEndpoint;

  console.log("🔧 Mock server test functions available:");
  console.log("   - window.testMockServer() - Test schema and resolvers");
  console.log("   - window.testLiveEndpoint() - Test live GraphQL endpoint");
}

// Run component tests immediately in development
if (import.meta.env?.DEV) {
  testMockServer().catch(console.error);
}
