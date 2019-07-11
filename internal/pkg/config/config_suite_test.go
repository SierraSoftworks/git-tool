package config_test

import (
    . "github.com/onsi/ginkgo"
    . "github.com/onsi/gomega"
    "github.com/onsi/ginkgo/reporters"
    "testing"
)

func TestConfig(t *testing.T) {
    RegisterFailHandler(Fail)
	junitReporter := reporters.NewJUnitReporter("junit.xml")
	RunSpecsWithDefaultAndCustomReporters(t, "Config Suite", []Reporter{junitReporter})
}