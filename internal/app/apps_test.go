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

var _ = Describe("gt apps", func() {
	var out *mocks.Output

	BeforeEach(func() {
		out = &mocks.Output{}
		di.SetOutput(out)
		di.SetConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))
	})

	It("Should be registered with the CLI", func() {
		Expect(app.NewApp().Command("apps")).ToNot(BeNil())
	})

	It("Should print out the list of apps which have been configured", func() {
		Expect(runApp("apps")).ToNot(HaveOccurred())

		Expect(out.GetOperations()).To(HaveLen(len(di.GetConfig().GetApps())))
	})

	It("Should print out every app", func() {
		Expect(runApp("apps")).ToNot(HaveOccurred())

		for _, app := range di.GetConfig().GetApps() {
			Expect(out.GetOperations()).To(ContainElement(app.Name() + "\n"))
		}
	})

	Context("Root autocompletion", func() {
		It("Should appear in the completions list", func() {
			Expect(runApp("complete", "gt")).ToNot(HaveOccurred())

			Expect(out.GetOperations()).To(ContainElement("apps\n"))
		})
	})

	Context("Command autocompletion", func() {
		It("Should return an empty completions list", func() {
			Expect(runApp("complete", "gt apps ")).ToNot(HaveOccurred())

			Expect(out.GetOperations()).To(BeEmpty())
		})
	})
})
