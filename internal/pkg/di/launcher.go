package di

import (
	"os"
	"os/exec"
	"os/signal"
)

var launcher Launcher = &defaultLauncher{}

func GetLauncher() Launcher {
	return launcher
}

func SetLauncher(l Launcher) {
	launcher = l
}

type Launcher interface {
	Run(cmd *exec.Cmd) error
}

func DefaultLauncher() Launcher {
	return &defaultLauncher{}
}

type defaultLauncher struct{}

func (l *defaultLauncher) Run(cmd *exec.Cmd) error {
	cmd.Stdin = os.Stdin
	cmd.Stderr = GetOutput()
	cmd.Stdout = GetOutput()

	sig := make(chan os.Signal, 1)
	signal.Notify(sig)

	go func() {
		for s := range sig {
			if cmd.Process != nil && cmd.ProcessState != nil && !cmd.ProcessState.Exited() {
				cmd.Process.Signal(s)
			}
		}
	}()

	defer func() {
		// Shutdown signal forwarding for our process
		signal.Stop(sig)
		close(sig)
	}()

	return cmd.Run()
}
