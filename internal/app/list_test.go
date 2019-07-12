package app_test

import (
	"github.com/SierraSoftworks/git-tool/internal/app"
	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/templates"
	"github.com/SierraSoftworks/git-tool/internal/pkg/mocks"
	"github.com/SierraSoftworks/git-tool/test"
	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
)

var _ = Describe("gt list", func() {
	var (
		out *mocks.Output
		err error
	)

	BeforeEach(func() {
		out = &mocks.Output{}
		di.SetOutput(out)
		di.SetConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))
	})

	It("Should be registered with the CLI", func() {
		Expect(app.NewApp().Command("list")).ToNot(BeNil())
	})

	It("Should print out the list of repos which have been configured", func() {
		Expect(runApp("list")).To(BeNil())

		repos, err := di.GetMapper().GetRepos()
		Expect(err).ToNot(HaveOccurred())

		Expect(out.GetOperations()).To(HaveLen(len(repos)))
	})

	Context("With no flags specified", func() {
		BeforeEach(func() {
			err = runApp("list")
		})

		It("Should not return an error", func() {
			Expect(err).ToNot(HaveOccurred())
		})

		It("Should print out every repo with its short info", func() {
			repos, err := di.GetMapper().GetRepos()
			Expect(err).ToNot(HaveOccurred())
			for _, r := range repos {
				Expect(out.GetOperations()).To(ContainElement(templates.RepoShortInfo(r) + "\n"))
			}
		})
	})

	Context("With the --quiet flag specified", func() {
		BeforeEach(func() {
			err = runApp("list", "--quiet")
		})

		It("Should not return an error", func() {
			Expect(err).ToNot(HaveOccurred())
		})

		It("Should print out every repo with its short info", func() {
			repos, err := di.GetMapper().GetRepos()
			Expect(err).ToNot(HaveOccurred())
			for _, r := range repos {
				Expect(out.GetOperations()).To(ContainElement(templates.RepoQualifiedName(r) + "\n"))
			}
		})
	})

	Context("With the --full flag specified", func() {
		BeforeEach(func() {
			err = runApp("list", "--full")
		})

		It("Should not return an error", func() {
			Expect(err).ToNot(HaveOccurred())
		})

		It("Should print out every repo with its short info", func() {
			repos, err := di.GetMapper().GetRepos()
			Expect(err).ToNot(HaveOccurred())
			for _, r := range repos {
				Expect(out.GetOperations()).To(ContainElement(templates.RepoFullInfo(r) + "\n"))
			}
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
			Expect(out.GetOperations()).To(ContainElement("list\n"))
		})
	})

	Context("Command autocompletion", func() {
		BeforeEach(func() {
			err = runApp("complete", "gt list ")
		})

		It("Should not return an error", func() {
			Expect(err).ToNot(HaveOccurred())
		})

		It("Should return an empty completions list", func() {
			Expect(out.GetOperations()).To(BeEmpty())
		})
	})
})
