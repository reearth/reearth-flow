import { graphql } from "@flow/lib/gql";

export const GET_ME = graphql(`
  query GetMe {
    me {
      id
      name
      email
    }
  }
`);
