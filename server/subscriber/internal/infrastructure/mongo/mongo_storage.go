package mongo

type MongoStorage struct {
	client          MongoClient
	jobCollection   string
	graphsCollection string
	edgesCollection  string
}

func NewMongoStorage(client MongoClient, jobCollection, graphsCollection, edgesCollection string) *MongoStorage {
	return &MongoStorage{
		client:          client,
		jobCollection:   jobCollection,
		graphsCollection: graphsCollection,
		edgesCollection:  edgesCollection,
	}
}
