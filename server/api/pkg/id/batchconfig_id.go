package id

type BatchConfigID struct{}

type BatchConfigIDBase = Base[BatchConfigID]

func NewBatchConfigID() BatchConfigIDBase {
	return NewID[BatchConfigID]()
}

func MustBatchConfigID(id string) BatchConfigIDBase {
	return Base[BatchConfigID]{}.Must(id)
}

func BatchConfigIDFrom(id string) (BatchConfigIDBase, error) {
	return Base[BatchConfigID]{}.From(id)
}

func BatchConfigIDFromRef(id *string) *BatchConfigIDBase {
	return Base[BatchConfigID]{}.FromRef(id)
}
