package app_test

import (
	"os"

	"github.com/SierraSoftworks/git-tool/internal/app"
	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/templates"
	"github.com/SierraSoftworks/git-tool/test"
	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
)

var _ = Describe("gt info", func() {
	var (
		out *di.TestOutput
		err error
	)

	BeforeEach(func() {
		out = &di.TestOutput{}
		di.SetOutput(out)
		di.SetConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))
	})

	It("Should be registered with the CLI", func() {
		Expect(app.NewApp().Command("info")).ToNot(BeNil())
	})

	Context("With no arguments", func() {
		BeforeEach(func() {
			os.Chdir(test.GetProjectRoot())
		})

		AfterEach(func() {
			os.Chdir(test.GetProjectRoot())
		})

		JustBeforeEach(func() {
			err = runApp("info")
		})

		Context("When not in a repository's directory", func() {
			It("Should return an error", func() {
				Expect(err).To(HaveOccurred())
			})

			It("Should inform the user in the error of why the command failed", func() {
				Expect(err.Error()).To(Equal("no repository specified"))
			})

			It("Should not print any output", func() {
				Expect(out.GetOperations()).To(BeEmpty())
			})
		})

		Context("When in a repository's directory", func() {
			BeforeEach(func() {
				os.Chdir(test.GetTestPath("devdir", "github.com", "sierrasoftworks", "test1"))
			})

			It("Should not return an error", func() {
				Expect(err).ToNot(HaveOccurred())
			})

			It("Should print information about the repository", func() {
				Expect(out.GetOperations()).To(HaveLen(1))

				repo, err := di.GetMapper().GetRepo("github.com/sierrasoftworks/test1")
				Expect(err).ToNot(HaveOccurred())
				Expect(out.GetOperations()[0]).To(Equal(templates.RepoFullInfo(repo) + "\n"))
			})
		})
	})

	Context("With a repository provided", func() {
		BeforeEach(func() {
			err = runApp("info", "github.com/sierrasoftworks/test1")
		})

		It("Should not return an error", func() {
			Expect(err).ToNot(HaveOccurred())
		})

		It("Should print information about the repository", func() {
			Expect(out.GetOperations()).To(HaveLen(1))

			repo, err := di.GetMapper().GetRepo("github.com/sierrasoftworks/test1")
			Expect(err).ToNot(HaveOccurred())
			Expect(out.GetOperations()[0]).To(Equal(templates.RepoFullInfo(repo) + "\n"))
		})
	})

	Context("With a bad repository name provided", func() {
		BeforeEach(func() {
			err = runApp("info", "badhubgit.orgcom/sierrasoftworks/missing")
		})

		It("Should return an error", func() {
			Expect(err).To(HaveOccurred())
		})

		It("Should inform the user in the error of why the command failed", func() {
			Expect(err.Error()).To(Equal("could not find repository"))
		})

		It("Should not print any output", func() {
			Expect(out.GetOperations()).To(BeEmpty())
		})
	})

	Context("Root autocompletion", func() {
		BeforeEach(func() {
			err = runApp("complete", "gt")
		})

		It("Should not return an error", func() {
			Expect(err).ToNot(HaveOccurred())
		})

		It("Should appear in the completions list", func() {
			Expect(out.GetOperations()).To(ContainElement("info\n"))
		})
	})

	Context("Command autocompletion", func() {
		BeforeEach(func() {
			err = runApp("complete", "gt info ")
		})

		It("Should not return an error", func() {
			Expect(err).ToNot(HaveOccurred())
		})

		It("Should return a completion list with the list of known repositories", func() {
			Expect(out.GetOperations()).ToNot(BeEmpty())
			Expect(out.GetOperations()).To(ContainElement("github.com/sierrasoftworks/test1\n"))
			Expect(out.GetOperations()).To(ContainElement("github.com/sierrasoftworks/test2\n"))
		})
	})
})
