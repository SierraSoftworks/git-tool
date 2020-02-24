package app_test

import (
	"path/filepath"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/mocks"
	"github.com/SierraSoftworks/git-tool/internal/pkg/registry"
	"github.com/SierraSoftworks/git-tool/test"
	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
)

var _ = Describe("gt config list", func() {
	var out *mocks.Output

	BeforeEach(func() {
		out = &mocks.Output{}
		di.SetOutput(out)
		di.SetConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))
		di.SetRegistry(registry.FileSystem(filepath.Join(test.GetProjectRoot(), "registry")))
	})

	It("Should print out the list of registry entries", func() {
		Expect(runApp("config", "list")).ToNot(HaveOccurred())

		Expect(len(out.GetOperations())).To(BeNumerically(">", 1))
		Expect(out.GetOperations()).To(ContainElement("services/github\n"))
		Expect(out.GetOperations()).To(ContainElement("apps/bash\n"))
	})

	Context("Root autocompletion", func() {
		It("Should appear in the completions list", func() {
			Expect(runApp("complete", "gt config ")).ToNot(HaveOccurred())

			Expect(out.GetOperations()).To(ContainElement("list\n"))
		})
	})
})
