package app_test

import (
	"github.com/SierraSoftworks/git-tool/internal/app"
)

func runApp(args ...string) error {
	return app.NewApp().Run(append([]string{
		"gt",
		"--config",
		"",
	}, args...))
}
