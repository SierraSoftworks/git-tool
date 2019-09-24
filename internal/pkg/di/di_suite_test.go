package di_test

import (
	"testing"

	"github.com/onsi/ginkgo/reporters"
	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
)

func TestDI(t *testing.T) {
	RegisterFailHandler(Fail)
	junitReporter := reporters.NewJUnitReporter("junit.xml")
	RunSpecsWithDefaultAndCustomReporters(t, "DI Suite", []Reporter{junitReporter})
}
