import { graphql } from "../__gen__";

graphql(`
  query GetMe {
    me {
      id
      name
      email
      myWorkspaceId
    }
  }
`);
