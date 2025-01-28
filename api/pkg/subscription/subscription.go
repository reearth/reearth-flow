package subscription

import (
	"sync"

	"github.com/reearth/reearth-flow/api/pkg/job"
)

type Manager struct {
	subscribers map[string][]chan job.Status
	mu          sync.RWMutex
}

func NewManager() *Manager {
	return &Manager{
		subscribers: make(map[string][]chan job.Status),
	}
}

func (m *Manager) Subscribe(jobID string) chan job.Status {
	ch := make(chan job.Status, 1)

	m.mu.Lock()
	m.subscribers[jobID] = append(m.subscribers[jobID], ch)
	m.mu.Unlock()

	return ch
}

func (m *Manager) Unsubscribe(jobID string, ch chan job.Status) {
	m.mu.Lock()
	defer m.mu.Unlock()

	subs := m.subscribers[jobID]
	for idx, sub := range subs {
		if sub == ch {
			close(sub)
			m.subscribers[jobID] = append(subs[:idx], subs[idx+1:]...)
			break
		}
	}
}

func (m *Manager) Notify(jobID string, status job.Status) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	for _, ch := range m.subscribers[jobID] {
		select {
		case ch <- status:
		default:
		}
	}
}
