package main

import (
	"os"

	"github.com/SierraSoftworks/git-tool/internal/app"
	"github.com/SierraSoftworks/sentry-go"
	"github.com/pkg/errors"
	"github.com/sirupsen/logrus"
)

var version = "0.0.1-dev"

func main() {
	sentry.AddDefaultOptions(
		sentry.DSN("https://eeccb0e19d324209a6ffd45ca98630a6@sentry.io/1486938"),
		sentry.Release(version),
	)

	raven := sentry.DefaultClient()

	defer func() {
		if r := recover(); r != nil {
			if err, ok := r.(error); ok {
				raven.Capture(
					sentry.ExceptionForError(err),
					sentry.Level(sentry.Fatal),
				).Wait()
			} else {
				raven.Capture(
					sentry.ExceptionForError(errors.Errorf("%v", r)),
					sentry.Level(sentry.Fatal),
				).Wait()
			}
		}
	}()

	app := app.NewApp()

	app.Version = version

	err := app.Run(os.Args)
	if err != nil {
		logrus.WithError(err).Error("Unexpected error occurred")
		raven.Capture(
			sentry.ExceptionForError(err),
			sentry.Level(sentry.Error),
		).Wait()
		os.Exit(1)
	}
}
