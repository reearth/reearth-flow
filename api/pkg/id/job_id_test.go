package id

import (
	"testing"

	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
)

func TestJob_Type(t *testing.T) {
	assert.Equal(t, "job", Job{}.Type())
}

func TestNewJobID(t *testing.T) {
	id1 := NewJobID()
	id2 := NewJobID()

	assert.NotEqual(t, id1, id2)
	assert.False(t, id1.IsNil())
	assert.False(t, id2.IsNil())
}

func TestJobIDFrom(t *testing.T) {
	tests := []struct {
		name    string
		input   string
		wantErr bool
	}{
		{
			name:    "valid uuid",
			input:   "123e4567-e89b-12d3-a456-426614174000",
			wantErr: false,
		},
		{
			name:    "invalid uuid",
			input:   "invalid-uuid",
			wantErr: true,
		},
		{
			name:    "empty string",
			input:   "",
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			id, err := JobIDFrom(tt.input)
			if tt.wantErr {
				assert.Error(t, err)
				assert.Equal(t, ErrInvalidID, err)
			} else {
				assert.NoError(t, err)
				assert.Equal(t, tt.input, id.String())
			}
		})
	}
}

func TestMustJobID(t *testing.T) {
	validUUID := "123e4567-e89b-12d3-a456-426614174000"

	t.Run("valid uuid", func(t *testing.T) {
		assert.NotPanics(t, func() {
			id := MustJobID(validUUID)
			assert.Equal(t, validUUID, id.String())
		})
	})

	t.Run("invalid uuid", func(t *testing.T) {
		assert.Panics(t, func() {
			MustJobID("invalid-uuid")
		})
	})
}

func TestJobIDFromRef(t *testing.T) {
	validUUID := "123e4567-e89b-12d3-a456-426614174000"

	t.Run("nil input", func(t *testing.T) {
		assert.Nil(t, JobIDFromRef(nil))
	})

	t.Run("valid uuid", func(t *testing.T) {
		input := validUUID
		result := JobIDFromRef(&input)
		assert.NotNil(t, result)
		assert.Equal(t, validUUID, result.String())
	})

	t.Run("invalid uuid", func(t *testing.T) {
		input := "invalid-uuid"
		assert.Nil(t, JobIDFromRef(&input))
	})
}

func TestJobID_String(t *testing.T) {
	u := uuid.New()
	id := JobID{id: u}
	assert.Equal(t, u.String(), id.String())
}

func TestJobID_GoString(t *testing.T) {
	u := uuid.New()
	id := JobID{id: u}
	assert.Equal(t, "JobID("+u.String()+")", id.GoString())
}

func TestJobID_IsNil(t *testing.T) {
	t.Run("nil pointer", func(t *testing.T) {
		var id *JobID
		assert.True(t, id.IsNil())
	})

	t.Run("nil uuid", func(t *testing.T) {
		id := &JobID{id: uuid.Nil}
		assert.True(t, id.IsNil())
	})

	t.Run("non-nil", func(t *testing.T) {
		id := &JobID{id: uuid.New()}
		assert.False(t, id.IsNil())
	})
}

func TestJobID_StringRef(t *testing.T) {
	t.Run("nil pointer", func(t *testing.T) {
		var id *JobID
		assert.Nil(t, id.StringRef())
	})

	t.Run("non-nil", func(t *testing.T) {
		u := uuid.New()
		id := &JobID{id: u}
		ref := id.StringRef()
		assert.NotNil(t, ref)
		assert.Equal(t, u.String(), *ref)
	})
}

func TestJobID_Ref(t *testing.T) {
	id := NewJobID()
	ref := id.Ref()
	assert.NotNil(t, ref)
	assert.Equal(t, id, *ref)
}

func TestJobID_Clone(t *testing.T) {
	original := NewJobID()
	clone := original.Clone()
	assert.Equal(t, original, clone)
}

func TestJobID_CloneRef(t *testing.T) {
	t.Run("nil pointer", func(t *testing.T) {
		var id *JobID
		assert.Nil(t, id.CloneRef())
	})

	t.Run("non-nil", func(t *testing.T) {
		original := NewJobID()
		clone := original.Ref().CloneRef()
		assert.NotNil(t, clone)
		assert.Equal(t, original, *clone)
	})
}

func TestJobIDListFrom(t *testing.T) {
	validUUID := "123e4567-e89b-12d3-a456-426614174000"

	t.Run("nil input", func(t *testing.T) {
		list, err := JobIDListFrom(nil)
		assert.NoError(t, err)
		assert.Nil(t, list)
	})

	t.Run("valid uuids", func(t *testing.T) {
		input := []string{validUUID, validUUID}
		list, err := JobIDListFrom(input)
		assert.NoError(t, err)
		assert.Len(t, list, 2)
		assert.Equal(t, validUUID, list[0].String())
		assert.Equal(t, validUUID, list[1].String())
	})

	t.Run("invalid uuid", func(t *testing.T) {
		input := []string{validUUID, "invalid-uuid"}
		list, err := JobIDListFrom(input)
		assert.Error(t, err)
		assert.Nil(t, list)
	})
}

func TestNewJobIDSet(t *testing.T) {
	set := NewJobIDSet()
	assert.NotNil(t, set)
	assert.Len(t, set, 0)
}
