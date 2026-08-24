package gql

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqldataloader"
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
)

type LogLoader struct {
	usecase interfaces.Log
}

func NewLogLoader(usecase interfaces.Log) *LogLoader {
	return &LogLoader{usecase: usecase}
}

func (l *LogLoader) GetLogs(ctx context.Context, since time.Time, jobID gqlmodel.ID) ([]*gqlmodel.Log, error) {
	newJobID, err := id.JobIDFrom(string(jobID))
	if err != nil {
		return nil, err
	}
	res, err := l.usecase.GetLogs(ctx, since, newJobID)
	if err != nil {
		return nil, err
	}

	logs := make([]*gqlmodel.Log, 0, len(res))
	for _, log := range res {
		logs = append(logs, gqlmodel.ToLog(log))
	}
	return logs, nil
}

// logsBatchKeySep separates the since-timestamp prefix from the job ID in a
// LogsByJobLoader key; NUL can't appear in either component.
const logsBatchKeySep = "\x00"

// LogsBatchKey builds the LogsByJobLoader key for a (jobID, since) pair. The
// same (obj, since) pair always resolves to the same field call within one
// GraphQL query, so this collapses to one key per distinct since value.
func LogsBatchKey(jobID gqlmodel.ID, since time.Time) string {
	return since.UTC().Format(time.RFC3339Nano) + logsBatchKeySep + string(jobID)
}

func parseLogsBatchKey(key string) (jobID id.JobID, since time.Time, err error) {
	sincePart, jobPart, ok := strings.Cut(key, logsBatchKeySep)
	if !ok {
		return id.JobID{}, time.Time{}, fmt.Errorf("malformed logs batch key: %q", key)
	}
	since, err = time.Parse(time.RFC3339Nano, sincePart)
	if err != nil {
		return id.JobID{}, time.Time{}, err
	}
	jobID, err = id.JobIDFrom(jobPart)
	if err != nil {
		return id.JobID{}, time.Time{}, err
	}
	return jobID, since, nil
}

// FetchByJobs is the batch fetch function for LogsByJobLoader. It groups keys
// by their since value (almost always a single group, since the GraphQL argument
// is shared across siblings) and issues one GetLogsBatch call per group. Every
// key gets an entry (empty slice if not found or not visible), so dataloaden's
// position-based matching stays correct.
func (l *LogLoader) FetchByJobs(ctx context.Context, keys []string) ([][]*gqlmodel.Log, []error) {
	type decoded struct {
		since time.Time
		jobID id.JobID
	}
	byPos := make([]decoded, len(keys))
	group := map[time.Time][]id.JobID{}

	for i, key := range keys {
		jid, since, err := parseLogsBatchKey(key)
		if err != nil {
			return nil, []error{err}
		}
		byPos[i] = decoded{jobID: jid, since: since}
		group[since] = append(group[since], jid)
	}

	logsByJob := map[id.JobID][]*log.Log{}
	for since, jobIDs := range group {
		res, err := l.usecase.GetLogsBatch(ctx, since, jobIDs)
		if err != nil {
			return nil, []error{err}
		}
		for jid, logs := range res {
			logsByJob[jid] = logs
		}
	}

	out := make([][]*gqlmodel.Log, len(keys))
	for i, d := range byPos {
		logs, ok := logsByJob[d.jobID]
		if !ok {
			continue
		}
		gqlLogs := make([]*gqlmodel.Log, 0, len(logs))
		for _, lg := range logs {
			gqlLogs = append(gqlLogs, gqlmodel.ToLog(lg))
		}
		out[i] = gqlLogs
	}

	return out, nil
}

// data loaders

type LogsByJobDataLoader interface {
	Load(string) ([]*gqlmodel.Log, error)
	LoadAll([]string) ([][]*gqlmodel.Log, []error)
}

func (l *LogLoader) ByJobDataLoader(ctx context.Context) LogsByJobDataLoader {
	return gqldataloader.NewLogsByJobLoader(gqldataloader.LogsByJobLoaderConfig{
		Wait:     dataLoaderWait,
		MaxBatch: dataLoaderMaxBatch,
		Fetch: func(keys []string) ([][]*gqlmodel.Log, []error) {
			return l.FetchByJobs(ctx, keys)
		},
	})
}

func (l *LogLoader) OrdinaryByJobDataLoader(ctx context.Context) LogsByJobDataLoader {
	return &ordinaryLogsByJobLoader{
		fetch: func(keys []string) ([][]*gqlmodel.Log, []error) {
			return l.FetchByJobs(ctx, keys)
		},
	}
}

type ordinaryLogsByJobLoader struct {
	fetch func(keys []string) ([][]*gqlmodel.Log, []error)
}

func (o *ordinaryLogsByJobLoader) Load(key string) ([]*gqlmodel.Log, error) {
	res, errs := o.fetch([]string{key})
	if len(errs) > 0 {
		return nil, errs[0]
	}
	if len(res) > 0 {
		return res[0], nil
	}
	return nil, nil
}

func (o *ordinaryLogsByJobLoader) LoadAll(keys []string) ([][]*gqlmodel.Log, []error) {
	return o.fetch(keys)
}
