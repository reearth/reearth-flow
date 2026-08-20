// k6 load test for the reearth-flow API GraphQL endpoint.
//
// Drives a realistic query/mutation mix (80% reads: list projects, list
// jobs; 20% writes: create + update a project) against a running server, so
// the request-latency-by-operation-name histogram and the accounts/Redis
// per-request instruments (see internal/app/otel) can be observed under
// load. Not run in CI — see ../README.md for how to run it locally.
import http from "k6/http";
import { check, sleep } from "k6";
import { Rate, Trend } from "k6/metrics";

const BASE_URL = __ENV.BASE_URL || "http://localhost:8080";
const TOKEN = __ENV.TOKEN || "";
const WORKSPACE_ID = __ENV.WORKSPACE_ID;

if (!WORKSPACE_ID) {
  throw new Error("WORKSPACE_ID env var is required, e.g. -e WORKSPACE_ID=...");
}

export const options = {
  scenarios: {
    graphql_mix: {
      executor: "ramping-vus",
      startVUs: 0,
      stages: [
        { duration: "30s", target: 10 },
        { duration: "2m", target: 10 },
        { duration: "30s", target: 0 },
      ],
    },
  },
  thresholds: {
    http_req_failed: ["rate<0.01"],
    http_req_duration: ["p(95)<1000"],
  },
};

const errorRate = new Rate("graphql_errors");
const opDuration = new Trend("graphql_operation_duration", true);

const headers = {
  "Content-Type": "application/json",
  ...(TOKEN ? { Authorization: `Bearer ${TOKEN}` } : {}),
};

const PAGINATION = { page: 1, pageSize: 20 };

const QUERIES = {
  ListProjects: `
    query ListProjects($workspaceId: ID!, $pagination: PageBasedPagination!) {
      projects(workspaceId: $workspaceId, pagination: $pagination) {
        totalCount
        nodes { id name updatedAt }
      }
    }
  `,
  ListJobs: `
    query ListJobs($workspaceId: ID!, $pagination: PageBasedPagination!) {
      jobs(workspaceId: $workspaceId, pagination: $pagination) {
        totalCount
        nodes { id status startedAt }
      }
    }
  `,
};

const MUTATIONS = {
  CreateProject: `
    mutation CreateProject($input: CreateProjectInput!) {
      createProject(input: $input) {
        project { id name }
      }
    }
  `,
  UpdateProject: `
    mutation UpdateProject($input: UpdateProjectInput!) {
      updateProject(input: $input) {
        project { id name updatedAt }
      }
    }
  `,
};

function graphql(operationName, query, variables) {
  const res = http.post(
    `${BASE_URL}/api/graphql`,
    JSON.stringify({ operationName, query, variables }),
    { headers, tags: { operation: operationName } },
  );

  const ok = check(res, {
    "status is 200": (r) => r.status === 200,
    "no graphql errors": (r) => {
      try {
        return !JSON.parse(r.body).errors;
      } catch (e) {
        return false;
      }
    },
  });
  errorRate.add(!ok);
  opDuration.add(res.timings.duration, { operation: operationName });
  return res;
}

export default function () {
  // Reads: 4 in 5 iterations.
  graphql("ListProjects", QUERIES.ListProjects, {
    workspaceId: WORKSPACE_ID,
    pagination: PAGINATION,
  });
  sleep(0.2);

  graphql("ListJobs", QUERIES.ListJobs, {
    workspaceId: WORKSPACE_ID,
    pagination: PAGINATION,
  });
  sleep(0.2);

  // Writes: 1 in 5 iterations.
  if (Math.random() < 0.2) {
    const created = graphql("CreateProject", MUTATIONS.CreateProject, {
      input: {
        workspaceId: WORKSPACE_ID,
        name: `k6-load-${__VU}-${__ITER}`,
        description: "created by k6 load test",
      },
    });

    let projectId;
    try {
      projectId = JSON.parse(created.body).data.createProject.project.id;
    } catch (e) {
      projectId = null;
    }

    if (projectId) {
      graphql("UpdateProject", MUTATIONS.UpdateProject, {
        input: { projectId, description: "updated by k6 load test" },
      });
    }
  }

  sleep(0.5);
}
