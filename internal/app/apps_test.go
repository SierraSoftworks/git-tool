package app

import (
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
)

var _ = Describe("gt apps", func() {
	var out *di.TestOutput
	app := NewApp()

	BeforeEach(func() {
		out = &di.TestOutput{}
		di.SetOutput(out)
	})

	It("Should be registered with the CLI", func() {
		Expect(app.Command("apps")).ToNot(BeNil())
	})

	It("Should print out the list of apps which have been configured", func() {
		Expect(app.Run([]string{
			"gt",
			"apps",
		})).To(BeNil())

		Expect(out.GetOperations()).To(HaveLen(len(di.GetConfig().GetApps())))
	})

	It("Should print out every app", func() {
		Expect(app.Run([]string{
			"gt",
			"apps",
		})).To(BeNil())

		for _, app := range di.GetConfig().GetApps() {
			Expect(out.GetOperations()).To(ContainElement(app.Name() + "\n"))
		}
	})

	Context("Root autocompletion", func() {
		It("Should appear in the completions list", func() {
			Expect(app.Run([]string{
				"gt",
				"complete",
				"gt",
			})).To(BeNil())

			Expect(out.GetOperations()).To(ContainElement("apps\n"))
		})
	})

	Context("Command autocompletion", func() {
		It("Should return an empty completions list", func() {
			Expect(app.Run([]string{
				"gt",
				"complete",
				"gt apps ",
			})).To(BeNil())

			Expect(out.GetOperations()).To(BeEmpty())
		})
	})
})
