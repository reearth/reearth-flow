package subscription

import (
	"fmt"
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

	subscribers := m.subscribers[jobID]
	droppedCount := 0

	for _, ch := range subscribers {
		select {
		case ch <- status:
			// Successfully sent
		default:
			// Channel is full, try to make room by removing oldest
			select {
			case <-ch:
				// Removed oldest, now try to send again
				select {
				case ch <- status:
					// Successfully sent after making room
				default:
					droppedCount++
				}
			default:
				droppedCount++
			}
		}
	}

	if droppedCount > 0 {
		// Log warning about dropped messages
		// Note: We can't use log package here due to circular dependency
		// This should be handled by the caller or consider using a callback
		fmt.Printf("WARNING: Dropped %d status notifications for job %s\n", droppedCount, jobID)
	}
}
