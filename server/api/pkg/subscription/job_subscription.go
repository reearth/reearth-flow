package subscription

import (
	"os"
	"sync"

	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/log"
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
	subscriberCount := len(m.subscribers[jobID])
	m.mu.Unlock()

	hostname, _ := os.Hostname()
	log.Debugf("[%s] New subscriber for job %s (total: %d)", hostname, jobID, subscriberCount)

	return ch
}

func (m *JobManager) Unsubscribe(jobID string, ch chan job.Status) {
	m.mu.Lock()
	defer m.mu.Unlock()

	beforeCount := len(m.subscribers[jobID])

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

	afterCount := len(m.subscribers[jobID])

	hostname, _ := os.Hostname()
	log.Debugf("[%s] Unsubscribed from job %s (subscribers: %d -> %d)", hostname, jobID, beforeCount, afterCount)
}

func (m *JobManager) Notify(jobID string, status job.Status) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	subscribers := m.subscribers[jobID]
	successCount := 0
	dropCount := 0

	for _, ch := range m.subscribers[jobID] {
		select {
		case ch <- status:
			successCount++
		default:
			// Log dropped message or implement retry logic
			// For now, we'll still drop but with larger buffer this should be rare
			dropCount++
			log.Warnf("[JobManager] Dropped status update for job %s: buffer full", jobID)
		}
	}

	hostname, _ := os.Hostname()
	log.Debugf("[%s] Notified job %s status %s to %d/%d subscribers (%d dropped)", hostname, jobID, status, successCount, len(subscribers), dropCount)
}
