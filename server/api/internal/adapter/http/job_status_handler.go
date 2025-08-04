package http

import (
	"encoding/json"
	"net/http"

	"github.com/gorilla/mux"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
)

type JobStatusHandler struct {
	jobUseCase JobUseCase
}

type JobUseCase interface {
	UpdateJobStatusFromEvent(jobID id.JobID, status job.Status) error
}

func NewJobStatusHandler(jobUseCase JobUseCase) *JobStatusHandler {
	return &JobStatusHandler{
		jobUseCase: jobUseCase,
	}
}

type JobStatusNotification struct {
	JobID  string `json:"jobId"`
	Status string `json:"status"`
}

func (h *JobStatusHandler) UpdateJobStatus(w http.ResponseWriter, r *http.Request) {
	vars := mux.Vars(r)
	jobIDStr := vars["jobId"]

	jobID, err := id.JobIDFrom(jobIDStr)
	if err != nil {
		http.Error(w, "Invalid job ID", http.StatusBadRequest)
		return
	}

	var notification JobStatusNotification
	if err := json.NewDecoder(r.Body).Decode(&notification); err != nil {
		http.Error(w, "Invalid request body", http.StatusBadRequest)
		return
	}

	if notification.JobID != jobIDStr {
		http.Error(w, "Job ID mismatch", http.StatusBadRequest)
		return
	}

	status := job.Status(notification.Status)
	if err := h.jobUseCase.UpdateJobStatusFromEvent(jobID, status); err != nil {
		http.Error(w, "Failed to update job status", http.StatusInternalServerError)
		return
	}

	w.WriteHeader(http.StatusOK)
	json.NewEncoder(w).Encode(map[string]string{"status": "ok"})
}
