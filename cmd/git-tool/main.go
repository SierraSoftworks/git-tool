package main

import (
	"os"
	"strings"

	"github.com/SierraSoftworks/git-tool/internal/app"
	"github.com/SierraSoftworks/sentry-go/v2"
	"github.com/pkg/errors"
	"github.com/sirupsen/logrus"
)

var version = "v0.0.1-dev"

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
		if strings.HasPrefix(err.Error(), "usage: ") {
			logrus.Error(err.Error()[len("usage: "):])
			os.Exit(1)
		}

		logrus.WithError(err).Error()
		raven.Capture(
			sentry.ExceptionForError(err),
			sentry.Level(sentry.Error),
		).Wait()
		os.Exit(1)
	}
}
