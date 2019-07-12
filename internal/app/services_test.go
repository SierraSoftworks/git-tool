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

var _ = Describe("gt services", func() {
	var out *mocks.Output

	BeforeEach(func() {
		out = &mocks.Output{}
		di.SetOutput(out)
		di.SetConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))
	})

	It("Should be registered with the CLI", func() {
		Expect(app.NewApp().Command("services")).ToNot(BeNil())
	})

	It("Should print out the list of services which have been configured", func() {
		Expect(runApp("services")).To(BeNil())

		Expect(out.GetOperations()).To(HaveLen(len(di.GetConfig().GetServices())))
	})

	It("Should print out every service", func() {
		Expect(runApp("services")).To(BeNil())

		for _, svc := range di.GetConfig().GetServices() {
			Expect(out.GetOperations()).To(ContainElement(svc.Domain() + "\n"))
		}
	})

	Context("Root autocompletion", func() {
		It("Should appear in the completions list", func() {
			Expect(runApp("complete", "gt")).To(BeNil())

			Expect(out.GetOperations()).To(ContainElement("services\n"))
		})
	})

	Context("Command autocompletion", func() {
		It("Should return an empty completions list", func() {
			Expect(runApp("complete", "gt services ")).To(BeNil())

			Expect(out.GetOperations()).To(BeEmpty())
		})
	})
})
