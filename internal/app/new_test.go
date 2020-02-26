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

var _ = Describe("gt new", func() {
	var (
		out  *mocks.Output
		init *mocks.Initializer
		err  error
	)

	BeforeEach(func() {
		out = &mocks.Output{}
		init = &mocks.Initializer{}
		di.SetOutput(out)
		di.SetInitializer(init)
		di.SetConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))
	})

	It("Should be registered with the CLI", func() {
		Expect(app.NewApp().Command("open")).ToNot(BeNil())
	})

	Context("With no arguments", func() {
		BeforeEach(func() {
			err = runApp("new")
		})

		It("Should return an error", func() {
			Expect(err).To(HaveOccurred())
		})

		It("Should inform the user in the error of why the command failed", func() {
			Expect(err.Error()).To(Equal("usage: no repository specified"))
		})

		It("Should not print any output", func() {
			Expect(out.GetOperations()).To(BeEmpty())
		})
	})

	Context("With a repository provided", func() {
		Context("When the repository doesn't exist locally", func() {
			BeforeEach(func() {
				err = runApp("new", "github.com/sierrasoftworks/git-tool")
			})

			It("Should not return an error", func() {
				Expect(err).ToNot(HaveOccurred())
			})

			It("Should not print any output", func() {
				Expect(out.GetOperations()).To(BeEmpty())
			})

			It("Should create the repository", func() {
				repo, err := di.GetMapper().GetRepo("github.com/sierrasoftworks/git-tool")
				Expect(err).ToNot(HaveOccurred())

				Expect(init.MockCalls).ToNot(BeEmpty())
				Expect(init.MockCalls[0].Function).To(Equal("CreateRepository"))
				Expect(init.MockCalls[0].Target.Path()).To(Equal(repo.Path()))
			})
		})

		Context("When the repository exists locally", func() {
			BeforeEach(func() {
				err = runApp("new", "github.com/sierrasoftworks/test1")
			})

			It("Should not return an error", func() {
				Expect(err).ToNot(HaveOccurred())
			})

			It("Should not print any output", func() {
				Expect(out.GetOperations()).To(BeEmpty())
			})

			It("Should attempt to create the repository", func() {
				repo, err := di.GetMapper().GetRepo("github.com/sierrasoftworks/test1")
				Expect(err).ToNot(HaveOccurred())

				Expect(init.MockCalls).ToNot(BeEmpty())
				Expect(init.MockCalls[0].Function).To(Equal("CreateRepository"))
				Expect(init.MockCalls[0].Target.Path()).To(Equal(repo.Path()))
			})
		})
	})

	Context("With a bad repository name provided", func() {
		BeforeEach(func() {
			err = runApp("new", "badhubgit.orgcom/sierrasoftworks/missing")
		})

		It("Should return an error", func() {
			Expect(err).To(HaveOccurred())
		})

		It("Should inform the user in the error of why the command failed", func() {
			Expect(err.Error()).To(Equal("usage: not a valid repository name"))
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
			Expect(out.GetOperations()).To(ContainElement("new\n"))
		})
	})

	Context("Command autocompletion", func() {
		BeforeEach(func() {
			err = runApp("complete", "gt new ")
		})

		It("Should not return an error", func() {
			Expect(err).ToNot(HaveOccurred())
		})

		It("Should return a completion list with the list of default namespaces", func() {
			Expect(out.GetOperations()).ToNot(BeEmpty())
			Expect(out.GetOperations()).To(ContainElement("spartan563/\n"))
			Expect(out.GetOperations()).To(ContainElement("sierrasoftworks/\n"))
		})

		It("Should return a completion list with the list of known namespaces", func() {
			Expect(out.GetOperations()).ToNot(BeEmpty())
			Expect(out.GetOperations()).To(ContainElement("github.com/spartan563/\n"))
			Expect(out.GetOperations()).To(ContainElement("github.com/sierrasoftworks/\n"))
			Expect(out.GetOperations()).To(ContainElement("dev.azure.com/sierrasoftworks/opensource/\n"))
		})
	})
})
