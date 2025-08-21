package gqlmodel

import "github.com/hasura/go-graphql-client"

type User struct {
	ID    graphql.ID     `json:"id" graphql:"id"`
	Name  graphql.String `json:"name" graphql:"name"`
	Email graphql.String `json:"email" graphql:"email"`
}
