package app_test

import (
	"fmt"
	"time"

	"github.com/SierraSoftworks/git-tool/internal/app"
	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/mocks"
	"github.com/SierraSoftworks/git-tool/test"
	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
)

var _ = Describe("gt scratch", func() {
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
		year, week := time.Now().UTC().ISOWeek()
		currentScratchpad := fmt.Sprintf("%dw%d", year, week)

		Context("When the scratchpad doesn't exist", func() {
			BeforeEach(func() {
				err = runApp("scratch")
			})

			It("Should not return an error", func() {
				Expect(err).ToNot(HaveOccurred())
			})

			It("Should not print any output", func() {
				Expect(out.GetOperations()).To(BeEmpty())
			})

			It("Should create the scratchpad", func() {
				repo, err := di.GetMapper().GetScratchpad(currentScratchpad)
				Expect(err).ToNot(HaveOccurred())

				Expect(init.MockCalls).ToNot(BeEmpty())
				Expect(init.MockCalls[0].Function).To(Equal("CreateScratchpad"))
				Expect(init.MockCalls[0].Target.Path()).To(Equal(repo.Path()))
			})

			It("Should launch the default app", func() {
				Expect(launch.GetCommands()).ToNot(BeEmpty())
				Expect(launch.GetCommands()).To(HaveLen(1))
				Expect(launch.GetCommands()[0].Args[0]).To(Equal("bash"))
			})

			It("Should launch the app in the scratch directory", func() {
				repo, err := di.GetMapper().GetScratchpad(currentScratchpad)
				Expect(err).ToNot(HaveOccurred())

				Expect(launch.GetCommands()).ToNot(BeEmpty())
				Expect(launch.GetCommands()).To(HaveLen(1))
				Expect(launch.GetCommands()[0].Dir).To(Equal(repo.Path()))
			})
		})
	})

	Context("With a week provided", func() {
		Context("When the scratchpad doesn't exist", func() {
			BeforeEach(func() {
				err = runApp("scratch", "2019w22")
			})

			It("Should not return an error", func() {
				Expect(err).ToNot(HaveOccurred())
			})

			It("Should not print any output", func() {
				Expect(out.GetOperations()).To(BeEmpty())
			})

			It("Should clone the repository", func() {
				repo, err := di.GetMapper().GetScratchpad("2019w22")
				Expect(err).ToNot(HaveOccurred())

				Expect(init.MockCalls).ToNot(BeEmpty())
				Expect(init.MockCalls[0].Function).To(Equal("CreateScratchpad"))
				Expect(init.MockCalls[0].Target.Path()).To(Equal(repo.Path()))
			})

			It("Should launch the default app", func() {
				Expect(launch.GetCommands()).ToNot(BeEmpty())
				Expect(launch.GetCommands()).To(HaveLen(1))
				Expect(launch.GetCommands()[0].Args[0]).To(Equal("bash"))
			})

			It("Should launch the app in the scratch directory", func() {
				repo, err := di.GetMapper().GetScratchpad("2019w22")
				Expect(err).ToNot(HaveOccurred())

				Expect(launch.GetCommands()).ToNot(BeEmpty())
				Expect(launch.GetCommands()).To(HaveLen(1))
				Expect(launch.GetCommands()[0].Dir).To(Equal(repo.Path()))
			})
		})

		Context("When the scratchpad already exists", func() {
			BeforeEach(func() {
				err = runApp("scratch", "2019w15")
			})

			It("Should not return an error", func() {
				Expect(err).ToNot(HaveOccurred())
			})

			It("Should not print any output", func() {
				Expect(out.GetOperations()).To(BeEmpty())
			})

			It("Should not re-create the scratchpad", func() {
				Expect(init.MockCalls).To(BeEmpty())
			})

			It("Should launch the default app", func() {
				Expect(launch.GetCommands()).ToNot(BeEmpty())
				Expect(launch.GetCommands()).To(HaveLen(1))
				Expect(launch.GetCommands()[0].Args[0]).To(Equal("bash"))
			})

			It("Should launch the app in the scratch directory", func() {
				repo, err := di.GetMapper().GetScratchpad("2019w15")
				Expect(err).ToNot(HaveOccurred())

				Expect(launch.GetCommands()).ToNot(BeEmpty())
				Expect(launch.GetCommands()).To(HaveLen(1))
				Expect(launch.GetCommands()[0].Dir).To(Equal(repo.Path()))
			})
		})
	})

	Context("With an app and scratchpad provided", func() {
		BeforeEach(func() {
			err = runApp("scratch", "shell", "2019w16")
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
			repo, err := di.GetMapper().GetScratchpad("2019w16")
			Expect(err).ToNot(HaveOccurred())

			Expect(launch.GetCommands()).ToNot(BeEmpty())
			Expect(launch.GetCommands()).To(HaveLen(1))
			Expect(launch.GetCommands()[0].Dir).To(Equal(repo.Path()))
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
			Expect(out.GetOperations()).To(ContainElement("scratch\n"))
		})
	})

	Context("Command autocompletion", func() {
		BeforeEach(func() {
			err = runApp("complete", "gt scratch ")
		})

		It("Should not return an error", func() {
			Expect(err).ToNot(HaveOccurred())
		})

		It("Should return a completion list with the list of known apps", func() {
			Expect(out.GetOperations()).ToNot(BeEmpty())
			Expect(out.GetOperations()).To(ContainElement("shell\n"))
		})

		It("Should return a completion list with the list of known scratchpads", func() {
			Expect(out.GetOperations()).ToNot(BeEmpty())
			Expect(out.GetOperations()).To(ContainElement("2019w15\n"))
			Expect(out.GetOperations()).To(ContainElement("2019w16\n"))
		})
	})
})
