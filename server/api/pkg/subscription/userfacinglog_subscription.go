package subscription

import (
	"sync"

	"github.com/reearth/reearth-flow/api/pkg/userfacinglog"
)

type UserFacingLogManager struct {
	subscribers map[string][]chan *userfacinglog.UserFacingLog
	mu          sync.RWMutex
}

func NewUserFacingLogManager() *UserFacingLogManager {
	return &UserFacingLogManager{
		subscribers: make(map[string][]chan *userfacinglog.UserFacingLog),
	}
}

func (m *UserFacingLogManager) Subscribe(key string) chan *userfacinglog.UserFacingLog {
	m.mu.Lock()
	defer m.mu.Unlock()

	ch := make(chan *userfacinglog.UserFacingLog, 100)
	m.subscribers[key] = append(m.subscribers[key], ch)
	return ch
}

func (m *UserFacingLogManager) Unsubscribe(key string, ch chan *userfacinglog.UserFacingLog) {
	m.mu.Lock()
	defer m.mu.Unlock()

	subs := m.subscribers[key]
	for i, subscriber := range subs {
		if subscriber == ch {
			m.subscribers[key] = append(subs[:i], subs[i+1:]...)
			close(ch)
			break
		}
	}

	if len(m.subscribers[key]) == 0 {
		delete(m.subscribers, key)
	}
}

func (m *UserFacingLogManager) Notify(key string, logs []*userfacinglog.UserFacingLog) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	for _, ch := range m.subscribers[key] {
		for _, l := range logs {
			select {
			case ch <- l:
			default:
			}
		}
	}
}

func (m *UserFacingLogManager) CountSubscribers(key string) int {
	m.mu.RLock()
	defer m.mu.RUnlock()
	return len(m.subscribers[key])
}
