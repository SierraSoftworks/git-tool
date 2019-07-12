package mocks

import (
	"os/exec"
)

type Launcher struct {
	commands []*exec.Cmd
}

func (l *Launcher) GetCommands() []*exec.Cmd {
	return l.commands
}

func (l *Launcher) Reset() {
	l.commands = []*exec.Cmd{}
}

func (l *Launcher) Run(cmd *exec.Cmd) error {
	l.commands = append(l.commands, cmd)
	return nil
}
