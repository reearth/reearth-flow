package gqlmodel

import "github.com/hasura/go-graphql-client"

type UserMetadata struct {
	Description graphql.String `json:"description" graphql:"description"`
	Lang        graphql.String `json:"lang" graphql:"lang"`
	PhotoURL    graphql.String `json:"photoURL" graphql:"photoURL"`
	Theme       graphql.String `json:"theme" graphql:"theme"`
	Website     graphql.String `json:"website" graphql:"website"`
}
