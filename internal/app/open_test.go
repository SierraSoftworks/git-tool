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

var _ = Describe("gt open", func() {
	var (
		out    *mocks.Output
		launch *mocks.Launcher
		init   *mocks.Initializer
		err    error
	)

	BeforeEach(func() {
		out = &mocks.Output{}
		launch = &mocks.Launcher{}
		init = &mocks.Initializer{}
		di.SetOutput(out)
		di.SetLauncher(launch)
		di.SetInitializer(init)
		di.SetConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))
	})

	It("Should be registered with the CLI", func() {
		Expect(app.NewApp().Command("open")).ToNot(BeNil())
	})

	Context("With no arguments", func() {
		BeforeEach(func() {
			err = runApp("open")
		})

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

	Context("With a repository provided", func() {
		Context("When the repository doesn't exist locally", func() {
			BeforeEach(func() {
				err = runApp("open", "github.com/sierrasoftworks/git-tool")
			})

			It("Should not return an error", func() {
				Expect(err).ToNot(HaveOccurred())
			})

			It("Should not print any output", func() {
				Expect(out.GetOperations()).To(BeEmpty())
			})

			It("Should clone the repository", func() {
				repo, err := di.GetMapper().GetRepo("github.com/sierrasoftworks/git-tool")
				Expect(err).ToNot(HaveOccurred())

				Expect(init.MockCalls).ToNot(BeEmpty())
				Expect(init.MockCalls[0].Function).To(Equal("Clone"))
				Expect(init.MockCalls[0].Repo.Path()).To(Equal(repo.Path()))
			})

			It("Should launch the default app", func() {
				Expect(launch.GetCommands()).ToNot(BeEmpty())
				Expect(launch.GetCommands()).To(HaveLen(1))
				Expect(launch.GetCommands()[0].Args[0]).To(Equal("bash"))
			})

			It("Should launch the app in the repo's directory", func() {
				repo, err := di.GetMapper().GetRepo("github.com/sierrasoftworks/git-tool")
				Expect(err).ToNot(HaveOccurred())

				Expect(launch.GetCommands()).ToNot(BeEmpty())
				Expect(launch.GetCommands()).To(HaveLen(1))
				Expect(launch.GetCommands()[0].Dir).To(Equal(repo.Path()))
			})
		})

		Context("When the repository exists locally", func() {
			BeforeEach(func() {
				err = runApp("open", "github.com/sierrasoftworks/test1")
			})

			It("Should not return an error", func() {
				Expect(err).ToNot(HaveOccurred())
			})

			It("Should not print any output", func() {
				Expect(out.GetOperations()).To(BeEmpty())
			})

			It("Should launch the default app", func() {
				Expect(launch.GetCommands()).ToNot(BeEmpty())
				Expect(launch.GetCommands()).To(HaveLen(1))
				Expect(launch.GetCommands()[0].Args[0]).To(Equal("bash"))
			})

			It("Should launch the app in the repo's directory", func() {
				repo, err := di.GetMapper().GetRepo("github.com/sierrasoftworks/test1")
				Expect(err).ToNot(HaveOccurred())

				Expect(launch.GetCommands()).ToNot(BeEmpty())
				Expect(launch.GetCommands()).To(HaveLen(1))
				Expect(launch.GetCommands()[0].Dir).To(Equal(repo.Path()))
			})
		})
	})

	Context("With an app and repository provided", func() {
		BeforeEach(func() {
			err = runApp("open", "shell", "github.com/sierrasoftworks/test1")
		})

		It("Should not return an error", func() {
			Expect(err).ToNot(HaveOccurred())
		})

		It("Should not print any output", func() {
			Expect(out.GetOperations()).To(BeEmpty())
		})

		It("Should launch the specified app", func() {
			Expect(launch.GetCommands()).ToNot(BeEmpty())
			Expect(launch.GetCommands()).To(HaveLen(1))
			Expect(launch.GetCommands()[0].Args[0]).To(Equal("bash"))
		})

		It("Should launch the app in the repo's directory", func() {
			repo, err := di.GetMapper().GetRepo("github.com/sierrasoftworks/test1")
			Expect(err).ToNot(HaveOccurred())

			Expect(launch.GetCommands()).ToNot(BeEmpty())
			Expect(launch.GetCommands()).To(HaveLen(1))
			Expect(launch.GetCommands()[0].Dir).To(Equal(repo.Path()))
		})
	})

	Context("With a bad repository name provided", func() {
		BeforeEach(func() {
			err = runApp("open", "badhubgit.orgcom/sierrasoftworks/missing")
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
			Expect(out.GetOperations()).To(ContainElement("open\n"))
		})
	})

	Context("Command autocompletion", func() {
		BeforeEach(func() {
			err = runApp("complete", "gt open ")
		})

		It("Should not return an error", func() {
			Expect(err).ToNot(HaveOccurred())
		})

		It("Should return a completion list with the list of known apps", func() {
			Expect(out.GetOperations()).ToNot(BeEmpty())
			Expect(out.GetOperations()).To(ContainElement("shell\n"))
		})

		It("Should return a completion list with the list of known repositories", func() {
			Expect(out.GetOperations()).ToNot(BeEmpty())
			Expect(out.GetOperations()).To(ContainElement("github.com/sierrasoftworks/test1\n"))
			Expect(out.GetOperations()).To(ContainElement("github.com/sierrasoftworks/test2\n"))
		})
	})
})
