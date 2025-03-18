package subscription

import (
	"sync"

	"github.com/reearth/reearth-flow/api/pkg/edge"
)

type EdgeManager struct {
	mu          sync.RWMutex
	subscribers map[string][]chan *edge.EdgeExecution
}

func NewEdgeManager() *EdgeManager {
	return &EdgeManager{
		subscribers: make(map[string][]chan *edge.EdgeExecution),
	}
}

func (m *EdgeManager) Subscribe(key string) chan *edge.EdgeExecution {
	ch := make(chan *edge.EdgeExecution, 50)
	m.mu.Lock()
	defer m.mu.Unlock()

	m.subscribers[key] = append(m.subscribers[key], ch)
	return ch
}

func (m *EdgeManager) Unsubscribe(key string, ch chan *edge.EdgeExecution) {
	m.mu.Lock()
	defer m.mu.Unlock()

	subs := m.subscribers[key]
	for i, sub := range subs {
		if sub == ch {
			close(sub)
			m.subscribers[key] = append(subs[:i], subs[i+1:]...)
			break
		}
	}

	if len(m.subscribers[key]) == 0 {
		delete(m.subscribers, key)
	}
}

func (m *EdgeManager) Notify(key string, edges []*edge.EdgeExecution) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	subs := m.subscribers[key]
	for _, e := range edges {
		for _, ch := range subs {
			select {
			case ch <- e:
			default:
			}
		}
	}
}

func (m *EdgeManager) NotifySingle(key string, e *edge.EdgeExecution) {
	if e == nil {
		return
	}

	m.Notify(key, []*edge.EdgeExecution{e})
}

func (m *EdgeManager) CountSubscribers(key string) int {
	m.mu.RLock()
	defer m.mu.RUnlock()

	return len(m.subscribers[key])
}
