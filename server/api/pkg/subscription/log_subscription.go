package subscription

import (
	"sync"

	"github.com/reearth/reearth-flow/api/pkg/log"
)

type LogManager struct {
	mu          sync.RWMutex
	subscribers map[string][]chan *log.Log
}

func NewLogManager() *LogManager {
	return &LogManager{
		subscribers: make(map[string][]chan *log.Log),
	}
}

func (m *LogManager) Subscribe(jobID string) chan *log.Log {
	ch := make(chan *log.Log, 50)
	m.mu.Lock()
	defer m.mu.Unlock()

	m.subscribers[jobID] = append(m.subscribers[jobID], ch)
	return ch
}

func (m *LogManager) Unsubscribe(jobID string, ch chan *log.Log) {
	m.mu.Lock()
	defer m.mu.Unlock()

	subs := m.subscribers[jobID]
	for i, sub := range subs {
		if sub == ch {
			close(sub)
			m.subscribers[jobID] = append(subs[:i], subs[i+1:]...)
			break
		}
	}

	if len(m.subscribers[jobID]) == 0 {
		delete(m.subscribers, jobID)
	}
}

func (m *LogManager) Notify(jobID string, logs []*log.Log) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	subs := m.subscribers[jobID]
	for _, l := range logs {
		for _, ch := range subs {
			select {
			case ch <- l:
			default:
			}
		}
	}
}

func (m *LogManager) CountSubscribers(jobID string) int {
	m.mu.RLock()
	defer m.mu.RUnlock()

	return len(m.subscribers[jobID])
}
