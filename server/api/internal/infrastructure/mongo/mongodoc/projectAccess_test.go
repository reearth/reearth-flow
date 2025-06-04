package mongodoc

import (
	"io"
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/projectAccess"

	"github.com/samber/lo"
	"github.com/stretchr/testify/assert"
	"go.mongodb.org/mongo-driver/bson"
)

func TestProjectAccessDocument_NewProjectAccessConsumer(t *testing.T) {
	paid := id.NewProjectAccessID()
	pid := id.NewProjectID()
	pa, _ := projectAccess.New().
		ID(paid).
		Project(pid).
		IsPublic(true).
		Token("token").
		Build()
	doc1, _ := NewProjectAccess(pa)
	p1 := lo.Must(bson.Marshal(doc1))

	tests := []struct {
		name    string
		arg     bson.Raw
		wantErr bool
		wantEOF bool
		result  []*projectAccess.ProjectAccess
	}{
		{
			name:    "consume project access",
			arg:     p1,
			wantErr: false,
			wantEOF: false,
			result:  []*projectAccess.ProjectAccess{pa},
		},
		{
			name:    "fail: unmarshal error",
			arg:     []byte{},
			wantErr: true,
			wantEOF: false,
			result:  nil,
		},
		{
			name:    "nil",
			arg:     nil,
			wantErr: false,
			wantEOF: true,
			result:  nil,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			c := NewProjectAccessConsumer()
			err := c.Consume(tc.arg)
			switch {
			case tc.wantEOF:
				assert.Equal(t, io.EOF, err)
			case tc.wantErr:
				assert.Error(t, err)
			default:
				assert.NoError(t, err)
			}
			assert.Equal(t, tc.result, c.Result)
		})
	}
}

func TestProjectAccessDocument_NewProjectAccess(t *testing.T) {
	paid := id.NewProjectAccessID()
	pid := id.NewProjectID()
	pa, _ := projectAccess.New().
		ID(paid).
		Project(pid).
		IsPublic(true).
		Token("token").
		Build()
	type args struct {
		pa *projectAccess.ProjectAccess
	}

	tests := []struct {
		name  string
		args  args
		want  *ProjectAccessDocument
		want1 string
	}{
		{
			name: "New project access",
			args: args{
				pa: pa,
			},
			want: &ProjectAccessDocument{
				ID:       paid.String(),
				Project:  pid.String(),
				IsPublic: true,
				Token:    "token",
			},
			want1: paid.String(),
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			got, got1 := NewProjectAccess(tc.args.pa)
			assert.Equal(t, tc.want1, got1)
			assert.Equal(t, tc.want, got)
		})
	}
}

func TestProjectAccessDocument_Model(t *testing.T) {
	paid := id.NewProjectAccessID()
	pid := id.NewProjectID()

	tests := []struct {
		name              string
		doc               *ProjectAccessDocument
		expectedID        id.ProjectAccessID
		expectedProjectID id.ProjectID
		exepectIsPublic   bool
		expectedToken     string
		expectErr         bool
	}{
		{
			name: "valid project access document",
			doc: &ProjectAccessDocument{
				ID:       paid.String(),
				Project:  pid.String(),
				IsPublic: true,
				Token:    "token",
			},
			expectedID:        paid,
			expectedProjectID: pid,
			exepectIsPublic:   true,
			expectedToken:     "token",
			expectErr:         false,
		},
		{
			name: "invalid project access ID",
			doc: &ProjectAccessDocument{
				ID:       "invalid-id",
				Project:  pid.String(),
				IsPublic: true,
				Token:    "token",
			},
			expectErr: true,
		},
		{
			name: "invalid project ID",
			doc: &ProjectAccessDocument{
				ID:       paid.String(),
				Project:  "invalid-id",
				IsPublic: true,
				Token:    "token",
			},
			expectErr: true,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			r, err := tc.doc.Model()

			if tc.expectErr {
				assert.Error(t, err)
				assert.Nil(t, r)
			} else {
				assert.NoError(t, err)
				if tc.doc != nil {
					assert.Equal(t, tc.expectedID, r.ID())
					assert.Equal(t, tc.expectedProjectID, r.Project())
					assert.Equal(t, tc.exepectIsPublic, r.IsPublic())
					assert.Equal(t, tc.expectedToken, r.Token())
				} else {
					assert.Nil(t, r)
				}
			}
		})
	}
}
