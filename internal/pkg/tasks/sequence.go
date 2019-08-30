package tasks

import "github.com/SierraSoftworks/git-tool/pkg/models"

// Sequence will run a series of tasks in sequence.
// It will stop execution of the sequence on the first error.
func Sequence(tasks ...Task) Task {
	return &sequenceTask{
		Tasks: tasks,
	}
}

// sequenceTask represents a sequence of tasks which will be executed in
// order. Execution will stop on the first error.
type sequenceTask struct {
	Tasks []Task
}

// ApplyRepo runs the task against a repository
func (t *sequenceTask) ApplyRepo(r models.Repo) error {
	for _, task := range t.Tasks {
		if err := task.ApplyRepo(r); err != nil {
			return err
		}
	}

	return nil
}

// ApplyScratchpad runs the task against a scratchpad
func (t *sequenceTask) ApplyScratchpad(r models.Scratchpad) error {
	for _, task := range t.Tasks {
		if err := task.ApplyScratchpad(r); err != nil {
			return err
		}
	}

	return nil
}
