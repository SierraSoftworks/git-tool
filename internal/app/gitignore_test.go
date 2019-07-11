package app

import (
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
)

var _ = Describe("gt ignore", func() {
	var (
		out *di.TestOutput
		err error
	)

	app := NewApp()

	BeforeEach(func() {
		out = &di.TestOutput{}
		di.SetOutput(out)
	})

	It("Should be registered with the CLI", func() {
		Expect(app.Command("ignore")).ToNot(BeNil())
	})

	Context("With no arguments", func() {
		BeforeEach(func() {
			err = app.Run([]string{
				"gt",
				"ignore",
			})
		})

		It("Should not return an error", func() {
			Expect(err).ToNot(HaveOccurred())
		})

		It("Should return the list of valid languages", func() {
			Expect(out.GetOperations()).ToNot(BeEmpty())
			Expect(out.GetOperations()).To(ContainElement(" - csharp\n"))
			Expect(out.GetOperations()).To(ContainElement(" - go\n"))
		})
	})

	Context("With a single language provided", func() {
		BeforeEach(func() {
			err = app.Run([]string{
				"gt",
				"ignore",
				"go",
			})
		})

		It("Should not return an error", func() {
			Expect(err).ToNot(HaveOccurred())
		})

		It("Should return the ignore file", func() {
			Expect(out.GetOperations()).To(HaveLen(1))
			Expect(out.GetOperations()[0]).To(ContainSubstring(".exe~"))
		})
	})

	Context("With multiple languages provded", func() {
		BeforeEach(func() {
			err = app.Run([]string{
				"gt",
				"ignore",
				"go",
				"node",
			})
		})

		It("Should not return an error", func() {
			Expect(err).ToNot(HaveOccurred())
		})

		It("Should return the ignore files", func() {
			Expect(out.GetOperations()).To(HaveLen(1))
			Expect(out.GetOperations()[0]).To(ContainSubstring(".exe~"))
			Expect(out.GetOperations()[0]).To(ContainSubstring("node_modules"))
		})
	})

	Context("Root autocompletion", func() {
		BeforeEach(func() {
			err = app.Run([]string{
				"gt",
				"complete",
				"gt",
			})
		})

		It("Should not return an error", func() {
			Expect(err).ToNot(HaveOccurred())
		})

		It("Should appear in the completions list", func() {
			Expect(out.GetOperations()).To(ContainElement("ignore\n"))
		})
	})

	Context("Command autocompletion", func() {
		BeforeEach(func() {
			err = app.Run([]string{
				"gt",
				"complete",
				"gt ignore ",
			})
		})

		It("Should not return an error", func() {
			Expect(err).ToNot(HaveOccurred())
		})

		It("Should return a completion list with the list of valid languages", func() {
			Expect(out.GetOperations()).ToNot(BeEmpty())
			Expect(out.GetOperations()).To(ContainElement("csharp\n"))
			Expect(out.GetOperations()).To(ContainElement("go\n"))
		})
	})
})
