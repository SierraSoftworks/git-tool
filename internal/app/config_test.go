package app_test

import (
	"github.com/SierraSoftworks/git-tool/internal/app"
	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/mocks"
	"github.com/SierraSoftworks/git-tool/test"
	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
)

var _ = Describe("gt config", func() {
	var out *mocks.Output

	BeforeEach(func() {
		out = &mocks.Output{}
		di.SetOutput(out)
		di.SetConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))
	})

	It("Should be registered with the CLI", func() {
		Expect(app.NewApp().Command("config")).ToNot(BeNil())
	})

	It("Should print out the current configuration file", func() {
		Expect(runApp("config")).ToNot(HaveOccurred())

		Expect(out.GetOperations()).To(HaveLen(1))
		Expect(out.GetOperations()[0]).To(ContainSubstring("apps:"))
		Expect(out.GetOperations()[0]).To(ContainSubstring("services:"))
	})

	Context("Root autocompletion", func() {
		It("Should appear in the completions list", func() {
			Expect(runApp("complete", "gt")).ToNot(HaveOccurred())

			Expect(out.GetOperations()).To(ContainElement("config\n"))
		})
	})

	Context("Command autocompletion", func() {
		It("Should return an empty completions list", func() {
			Expect(runApp("complete", "gt config ")).ToNot(HaveOccurred())

			Expect(out.GetOperations()).To(BeEmpty())
		})
	})
})
