package gqlmodel

import (
	"github.com/reearth/reearthx/idx"
)

type ID string

func IDFrom[T idx.Type](i idx.ID[T]) ID {
	return ID(i.String())
}

func IDFromList[T idx.Type](i idx.List[T]) []ID {
	if i == nil {
		return nil
	}
	res := make([]ID, len(i))
	for i, v := range i {
		res[i] = IDFrom[T](v)
	}
	return res
}

func IDFromRef[T idx.Type](i *idx.ID[T]) *ID {
	return (*ID)(i.StringRef())
}

func IDFromString[T idx.Type](i idx.StringID[T]) ID {
	return (ID)(i)
}

func IDFromStringRef[T idx.Type](i *idx.StringID[T]) *ID {
	return (*ID)(i)
}

func ToID[A idx.Type](a ID) (idx.ID[A], error) {
	return idx.From[A](string(a))
}

func ToIDs[A idx.Type](a []ID) (*[]idx.ID[A], error) {
	if a == nil {
		return nil, nil
	}
	res := make([]idx.ID[A], len(a))
	for i, v := range a {
		r, err := ToID[A](v)
		if err != nil {
			return nil, err
		}
		res[i] = r
	}
	return &res, nil
}

func ToID2[A, B idx.Type](a, b ID) (ai idx.ID[A], bi idx.ID[B], err error) {
	ai, err = ToID[A](a)
	if err != nil {
		return
	}
	bi, err = ToID[B](b)
	return
}

func ToID3[A, B, C idx.Type](a, b, c ID) (ai idx.ID[A], bi idx.ID[B], ci idx.ID[C], err error) {
	ai, bi, err = ToID2[A, B](a, b)
	if err != nil {
		return
	}
	ci, err = ToID[C](c)
	return
}

func ToID4[A, B, C, D idx.Type](a, b, c, d ID) (ai idx.ID[A], bi idx.ID[B], ci idx.ID[C], di idx.ID[D], err error) {
	ai, bi, err = ToID2[A, B](a, b)
	if err != nil {
		return
	}
	ci, di, err = ToID2[C, D](c, d)
	return
}

func ToIDRef[A idx.Type](a *ID) *idx.ID[A] {
	return idx.FromRef[A]((*string)(a))
}

func ToStringIDRef[T idx.Type](a *ID) *idx.StringID[T] {
	return idx.StringIDFromRef[T]((*string)(a))
}
