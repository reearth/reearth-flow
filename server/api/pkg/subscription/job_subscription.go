package subscription

import (
	"sync"

	"github.com/reearth/reearth-flow/api/pkg/job"
)

type JobManager struct {
	subscribers map[string][]chan job.Status
	mu          sync.RWMutex
}

func NewJobManager() *JobManager {
	return &JobManager{
		subscribers: make(map[string][]chan job.Status),
	}
}

func (m *JobManager) CountSubscribers(jobID string) int {
	m.mu.RLock()
	defer m.mu.RUnlock()

	return len(m.subscribers[jobID])
}

func (m *JobManager) Subscribe(jobID string) chan job.Status {
	ch := make(chan job.Status, 50)

	m.mu.Lock()
	m.subscribers[jobID] = append(m.subscribers[jobID], ch)
	m.mu.Unlock()

	return ch
}

func (m *JobManager) Unsubscribe(jobID string, ch chan job.Status) {
	m.mu.Lock()
	defer m.mu.Unlock()

	subs := m.subscribers[jobID]
	for idx, sub := range subs {
		if sub == ch {
			close(sub)
			m.subscribers[jobID] = append(subs[:idx], subs[idx+1:]...)
			if len(m.subscribers[jobID]) == 0 {
				delete(m.subscribers, jobID)
			}
			break
		}
	}
}

func (m *JobManager) Notify(jobID string, status job.Status) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	for _, ch := range m.subscribers[jobID] {
		select {
		case ch <- status:
		default:
			// Log dropped message or implement retry logic
			// For now, we'll still drop but with larger buffer this should be rare
		}
	}
}
