package app_test

import (
	"os"

	"github.com/SierraSoftworks/git-tool/internal/app"
	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/mocks"
	"github.com/SierraSoftworks/git-tool/internal/pkg/tasks"
	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/SierraSoftworks/git-tool/test"
	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
)

var _ = Describe("gt branch", func() {
	var (
		out  *mocks.Output
		repo models.Repo
		err  error
	)

	BeforeEach(func() {
		out = &mocks.Output{}
		di.SetOutput(out)
		di.SetConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))

		repo, err = di.GetMapper().GetFullyQualifiedRepo("github.com/sierrasoftworks/branch_test_repo")
		if err != nil {
			return
		}

		err = tasks.Sequence(
			tasks.NewFolder(),
			tasks.GitInit(),
			tasks.NewFile("README.md", []byte("# Test Repo")),
			tasks.GitCommit("Initial Commit", "README.md"),
			tasks.GitCheckout("master"),
		).ApplyRepo(repo)
		if err != nil {
			return
		}

		err = os.Chdir(repo.Path())
		if err != nil {
			return
		}
	})

	It("Should not encounter any errors setting up the environment", func() {
		Expect(err).ToNot(HaveOccurred())
	})

	AfterEach(func() {
		os.Chdir(test.GetProjectRoot())
		os.RemoveAll(repo.Path())
	})

	It("Should be registered with the CLI", func() {
		Expect(app.NewApp().Command("branch")).ToNot(BeNil())
	})

	It("Should return an error if no arguments are provided", func() {
		Expect(runApp("branch")).To(HaveOccurred())
	})

	Context("Root autocompletion", func() {
		It("Should appear in the completions list", func() {
			Expect(runApp("complete", "gt ")).ToNot(HaveOccurred())

			Expect(out.GetOperations()).To(ContainElement("branch\n"))
		})
	})

	Context("Command autocompletion", func() {
		It("Should return a list of branches", func() {
			Expect(runApp("complete", "gt branch ")).ToNot(HaveOccurred())

			Expect(out.GetOperations()).ToNot(BeEmpty())
			Expect(out.GetOperations()).To(ContainElement("master\n"))
		})
	})
})
