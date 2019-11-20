package app_test

import (
	"github.com/SierraSoftworks/git-tool/internal/app"
	"github.com/SierraSoftworks/git-tool/internal/pkg/autocomplete"
	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/mocks"
	"github.com/SierraSoftworks/git-tool/test"
	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
)

var _ = Describe("gt shell-init", func() {
	var out *mocks.Output

	BeforeEach(func() {
		out = &mocks.Output{}
		di.SetOutput(out)
		di.SetConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))
	})

	It("Should be registered with the CLI", func() {
		Expect(app.NewApp().Command("shell-init")).ToNot(BeNil())
	})

	It("Should print out the list of shells if there is no shell provided", func() {
		Expect(runApp("shell-init")).To(BeNil())

		Expect(out.GetOperations()).To(HaveLen(len(autocomplete.GetInitScriptShells())))
	})

	It("Should print out nothing if the shell is not found", func() {
		Expect(runApp("shell-init", "unrecognized")).To(BeNil())

		Expect(out.GetOperations()).To(BeEmpty())
	})

	Context("bash", func() {
		It("should return the bash init script", func() {
			Expect(runApp("shell-init", "bash")).To(BeNil())

			Expect(out.GetOperations()).To(ContainElement(autocomplete.GetInitScript("bash")))
		})
	})

	Context("powershell", func() {
		It("should return the powershell init script", func() {
			Expect(runApp("shell-init", "powershell")).To(BeNil())

			Expect(out.GetOperations()).To(ContainElement(autocomplete.GetInitScript("powershell")))
		})
	})

	Context("zsh", func() {
		It("should return the zsh init script", func() {
			Expect(runApp("shell-init", "zsh")).To(BeNil())

			Expect(out.GetOperations()).To(ContainElement(autocomplete.GetInitScript("zsh")))
		})
	})

	Context("Root completion", func() {
		It("Should appear in the completions list", func() {
			Expect(runApp("complete", "shell-init")).To(BeNil())

			Expect(out.GetOperations()).To(ContainElement("shell-init\n"))
		})
	})

	Context("Command autocompletion", func() {
		It("Should return the list of shells", func() {
			Expect(runApp("complete", "gt shell-init ")).To(BeNil())

			for _, shell := range autocomplete.GetInitScriptShells() {
				Expect(out.GetOperations()).To(ContainElement(shell + "\n"))
			}
		})
	})
})
