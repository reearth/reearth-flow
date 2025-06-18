package subscription

import (
	"sync"

	"github.com/reearth/reearth-flow/api/pkg/graph"
)

type NodeManager struct {
	mu          sync.RWMutex
	subscribers map[string][]chan *graph.NodeExecution
}

func NewNodeManager() *NodeManager {
	return &NodeManager{
		subscribers: make(map[string][]chan *graph.NodeExecution),
	}
}

func (m *NodeManager) Subscribe(key string) chan *graph.NodeExecution {
	ch := make(chan *graph.NodeExecution, 50)
	m.mu.Lock()
	defer m.mu.Unlock()

	m.subscribers[key] = append(m.subscribers[key], ch)
	return ch
}

func (m *NodeManager) Unsubscribe(key string, ch chan *graph.NodeExecution) {
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

func (m *NodeManager) Notify(key string, edges []*graph.NodeExecution) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	subs := m.subscribers[key]
	for _, e := range edges {
		for _, ch := range subs {
			select {
			case ch <- e:
			default:
				// Log dropped message or implement retry logic
				// For now, we'll still drop but with larger buffer this should be rare
			}
		}
	}
}

func (m *NodeManager) NotifySingle(key string, e *graph.NodeExecution) {
	if e == nil {
		return
	}

	m.Notify(key, []*graph.NodeExecution{e})
}

func (m *NodeManager) CountSubscribers(key string) int {
	m.mu.RLock()
	defer m.mu.RUnlock()

	return len(m.subscribers[key])
}
