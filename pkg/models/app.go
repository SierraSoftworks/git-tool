package models

import (
	"os/exec"
)

// An app represents an application which can be started
// within the context of a Target.
type App interface {
	Name() string

	GetCmd(target Target) (*exec.Cmd, error)
}
