package app_test

import (
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/app"
	. "github.com/onsi/ginkgo"
	"github.com/onsi/ginkgo/reporters"
	. "github.com/onsi/gomega"
)

func TestApp(t *testing.T) {
	RegisterFailHandler(Fail)
	junitReporter := reporters.NewJUnitReporter("junit.xml")
	RunSpecsWithDefaultAndCustomReporters(t, "App Suite", []Reporter{junitReporter})
}

func runApp(args ...string) error {
	return app.NewApp().Run(append([]string{
		"gt",
		"--config",
		"",
	}, args...))
}
