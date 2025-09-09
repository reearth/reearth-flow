package monitor

import (
	"os"
	"sync"

	"github.com/reearth/reearthx/log"
)

type Monitor struct {
	configs map[string]*Config
	mu      sync.RWMutex
}

func NewMonitor() *Monitor {
	return &Monitor{
		configs: make(map[string]*Config),
	}
}

func (m *Monitor) Register(jobID string, config *Config) {
	m.mu.Lock()
	defer m.mu.Unlock()

	if oldConfig, exists := m.configs[jobID]; exists {
		oldConfig.Cancel()
		hostname, _ := os.Hostname()
		log.Debugf("[%s] Monitor: Replaced existing config for job %s", hostname, jobID)
	}
	m.configs[jobID] = config
}

func (m *Monitor) Get(jobID string) *Config {
	m.mu.RLock()
	defer m.mu.RUnlock()
	return m.configs[jobID]
}

func (m *Monitor) Remove(jobID string) {
	m.mu.Lock()
	defer m.mu.Unlock()

	if config, exists := m.configs[jobID]; exists {
		config.Cancel()
		delete(m.configs, jobID)
		hostname, _ := os.Hostname()
		log.Debugf("[%s] Monitor: Removed config for job %s", hostname, jobID)
	}
}
