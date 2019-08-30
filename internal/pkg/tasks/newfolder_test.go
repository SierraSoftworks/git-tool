package tasks_test

import (
	"os"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/mocks"
	"github.com/SierraSoftworks/git-tool/internal/pkg/repo"
	"github.com/SierraSoftworks/git-tool/internal/pkg/tasks"
	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/SierraSoftworks/git-tool/test"
	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
)

var _ = Describe("New Folder Task", func() {
	var (
		out    *mocks.Output
		launch *mocks.Launcher
		r      models.Repo
		sp     models.Scratchpad
		cfg    *mocks.Config
		err    error
	)

	BeforeEach(func() {
		cfg = mocks.NewConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))
		out = &mocks.Output{}
		launch = &mocks.Launcher{}
		di.SetOutput(out)
		di.SetLauncher(launch)
		di.SetMapper(&repo.Mapper{})
		di.SetInitializer(&repo.Initializer{})
		di.SetConfig(cfg)

		r = repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test3")
		sp = repo.NewScratchpad("2019w28")
	})

	AfterEach(func() {
		os.RemoveAll(r.Path())
		os.RemoveAll(sp.Path())
		os.Chdir(test.GetProjectRoot())
	})

	Describe("NewFolder()", func() {
		Context("when applied to a repo", func() {
			JustBeforeEach(func() {
				err = tasks.NewFolder().ApplyRepo(r)
			})

			Context("which doesn't exist", func() {

				It("Should not return an error", func() {
					Expect(err).ToNot(HaveOccurred())
				})

				It("Should have created the repo folder", func() {
					Expect(r.Exists()).To(BeTrue())
				})
			})

			Context("which doesn't exist", func() {
				BeforeEach(func() {
					os.MkdirAll(r.Path(), os.ModePerm)
				})

				It("Should not return an error", func() {
					Expect(err).ToNot(HaveOccurred())
				})

				It("Should have created the repo folder", func() {
					Expect(r.Exists()).To(BeTrue())
				})
			})
		})

		Context("when applied to a scratchpad", func() {
			JustBeforeEach(func() {
				err = tasks.NewFolder().ApplyScratchpad(sp)
			})

			Context("which doesn't exist", func() {
				BeforeEach(func() {
					os.MkdirAll(sp.Path(), os.ModePerm)
				})

				It("Should not return an error", func() {
					Expect(err).ToNot(HaveOccurred())
				})

				It("Should have created the scratchpad folder", func() {
					Expect(sp.Exists()).To(BeTrue())
				})
			})
		})
	})
})
