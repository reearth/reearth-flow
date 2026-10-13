import { makeExecutableSchema } from "@graphql-tools/schema";
import { graphql as executeGraphQL } from "graphql";
import { graphql } from "msw";

import { resolvers } from "../schema/resolvers";
import { typeDefs } from "../schema/typeDefs";

// Create executable schema
const schema = makeExecutableSchema({
  typeDefs,
  resolvers,
});

// Helper function to execute GraphQL operations
const executeGraphQLOperation = async (
  operation: string,
  variables?: Record<string, any>,
  context?: any,
) => {
  try {
    const result = await executeGraphQL({
      schema,
      source: operation,
      variableValues: variables,
      contextValue: context,
    });

    if (result.errors) {
      console.error("GraphQL Errors:", result.errors);
      return {
        errors: result.errors.map((error) => ({
          message: error.message,
          locations: error.locations,
          path: error.path,
        })),
        data: result.data,
      };
    }

    return {
      data: result.data,
    };
  } catch (error) {
    console.error("GraphQL Execution Error:", error);
    return {
      errors: [
        {
          message:
            error instanceof Error ? error.message : "Internal server error",
        },
      ],
    };
  }
};

// Extract Authorization token from headers
const getAuthContext = (request: Request) => {
  const authHeader = request.headers.get("Authorization");
  const token = authHeader?.replace("Bearer ", "");

  return {
    token,
    isAuthenticated: !!token,
  };
};

export const graphqlHandlers = [
  graphql
    .link("*/api/graphql")
    .operation(async ({ request, variables, operationName }) => {
      console.log("🚀 GraphQL Operation:", operationName || "Unknown");
      console.log("📝 Variables:", variables);

      // Get authentication context
      const authContext = getAuthContext(request);

      // For non-authenticated requests, return auth error for protected operations
      if (
        !authContext.isAuthenticated &&
        operationName !== "IntrospectionQuery"
      ) {
        return new Response(
          JSON.stringify({
            errors: [
              {
                message: "Authentication required",
                extensions: { code: "UNAUTHENTICATED" },
              },
            ],
          }),
          {
            status: 401,
            headers: { "Content-Type": "application/json" },
          },
        );
      }

      // Get the raw query from the request
      const requestBody = (await request.json()) as {
        query: string;
        variables?: any;
        operationName?: string;
      } | null;

      if (!requestBody) {
        return new Response(
          JSON.stringify({
            errors: [{ message: "Invalid request body" }],
          }),
          { status: 400, headers: { "Content-Type": "application/json" } },
        );
      }

      const { query, variables: requestVariables } = requestBody;

      // Execute the GraphQL operation
      const result = await executeGraphQLOperation(
        query,
        requestVariables || variables,
        authContext,
      );

      console.log("✅ GraphQL Result:", result);

      return new Response(JSON.stringify(result), {
        status: 200,
        headers: {
          "Content-Type": "application/json",
          "Access-Control-Allow-Origin": "*",
          "Access-Control-Allow-Headers": "Content-Type, Authorization",
          "Access-Control-Allow-Methods": "POST, OPTIONS",
        },
      });
    }),
];
