package app_test

import (
	"fmt"
	"io/ioutil"
	"os"
	"path/filepath"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/mocks"
	"github.com/SierraSoftworks/git-tool/internal/pkg/registry"
	"github.com/SierraSoftworks/git-tool/test"
	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
)

var _ = Describe("gt config add", func() {
	var out *mocks.Output

	BeforeEach(func() {
		out = &mocks.Output{}
		di.SetOutput(out)
		di.SetConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))
		di.SetRegistry(registry.FileSystem(filepath.Join(test.GetProjectRoot(), "registry")))
	})

	BeforeEach(func() {
		data, _ := ioutil.ReadFile(test.GetTestDataPath("config.updated.yaml"))
		ioutil.WriteFile(test.GetTestDataPath("config.updated.yaml"), data, os.ModePerm)
	})

	AfterEach(func() {
		os.Remove(test.GetTestDataPath("config.updated.yaml"))
	})

	It("Should return an error if no arguments are provided", func() {
		Expect(runApp("config", "add")).To(HaveOccurred())
	})

	It("Should return an error if no config file path is provided", func() {
		Expect(runApp(fmt.Sprintf("--config=%s", ""), "config", "add")).To(HaveOccurred())
	})

	It("Should not return an error if a config file is provided", func() {
		Expect(runApp(fmt.Sprintf("--config=%s", test.GetTestDataPath("config.updated.yaml")), "config", "add", "apps/bash")).ToNot(HaveOccurred())

		Expect(out.GetOperations()).ToNot(BeEmpty())
		Expect(out.GetOperations()).To(ContainElement("Applying apps/bash\n"))

		cfg, err := config.Load(test.GetTestDataPath("config.updated.yaml"))
		Expect(err).ToNot(HaveOccurred())
		Expect(cfg).ToNot(BeNil())
		Expect(cfg.GetApp("bash")).ToNot(BeNil())
	})

	Context("Root autocompletion", func() {
		It("Should appear in the completions list", func() {
			Expect(runApp("complete", "gt config ")).ToNot(HaveOccurred())

			Expect(out.GetOperations()).To(ContainElement("add\n"))
		})
	})

	Context("Command autocompletion", func() {
		It("Should return a series of valid prefixes for templates to add", func() {
			Expect(runApp("complete", "gt config add ")).ToNot(HaveOccurred())

			Expect(out.GetOperations()).ToNot(BeEmpty())
			Expect(out.GetOperations()).To(ContainElement("apps/\n"))
			Expect(out.GetOperations()).To(ContainElement("services/\n"))
		})
	})
})
