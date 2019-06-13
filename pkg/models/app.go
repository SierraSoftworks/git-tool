package models

import (
	"os/exec"
)

// An app represents an application which can be started
// within the context of a repository.
type App interface {
	Name() string

	GetCmd(repo Repo) (*exec.Cmd, error)
}