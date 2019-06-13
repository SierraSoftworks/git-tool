package main

import (
	"os"

	"github.com/SierraSoftworks/git-tool/internal/app"
	"github.com/SierraSoftworks/sentry-go"
	"github.com/sirupsen/logrus"
)

var version = "0.0.1-dev"

func main() {
	raven := sentry.DefaultClient()

	app := app.NewApp()

	app.Version = version

	err := app.Run(os.Args)
	if err != nil {
		logrus.WithError(err).Error("Unexpected error occurred")
		raven.Capture(
			sentry.ExceptionForError(err),
			sentry.Level(sentry.Fatal),
		).Wait()
		os.Exit(1)
	}
}
