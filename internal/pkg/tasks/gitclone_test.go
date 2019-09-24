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

var _ = Describe("Git Clone Task", func() {
	var (
		out *mocks.Output
		r   models.Repo
		sp  models.Scratchpad
		cfg *mocks.Config
		err error
	)

	BeforeEach(func() {
		cfg = mocks.NewConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))
		out = &mocks.Output{}
		di.SetOutput(out)
		di.SetLauncher(di.DefaultLauncher())
		di.SetMapper(&repo.Mapper{})
		di.SetInitializer(&repo.Initializer{})
		di.SetConfig(cfg)

		r = repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test1")
		sp = repo.NewScratchpad("2019w15")
	})

	AfterEach(func() {
		os.Chdir(test.GetProjectRoot())
	})

	Describe("GitClone()", func() {
		runCloneTests := func() {
			Context("when applied to a repo", func() {
				JustBeforeEach(func() {
					err = tasks.GitClone().ApplyRepo(r)
				})

				Context("which doesn't exist remotely", func() {
					BeforeEach(func() {
						r = repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test3")
					})

					AfterEach(func() {
						os.RemoveAll(r.Path())
					})

					It("Should return an error", func() {
						Expect(err).To(HaveOccurred())
					})

					It("Should not have created the repo folder", func() {
						Expect(r.Exists()).To(BeFalse())
					})
				})

				Context("which doesn't exist locally", func() {
					BeforeEach(func() {
						r = repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/licenses")
					})

					AfterEach(func() {
						os.RemoveAll(r.Path())
					})

					It("should log the clone progress", func() {
						Expect(out.GetOperations()).ToNot(BeEmpty())
					})

					It("Should not return an error", func() {
						Expect(err).ToNot(HaveOccurred())
					})

					It("Should have created the repo folder", func() {
						Expect(r.Exists()).To(BeTrue())
					})
				})

				Context("which exists locally", func() {
					BeforeEach(func() {
						r = repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test1")
					})

					It("should not log anything", func() {
						Expect(out.GetOperations()).To(BeEmpty())
					})

					It("Should not return an error", func() {
						Expect(err).ToNot(HaveOccurred())
					})

					It("Should still have the local repo folder", func() {
						Expect(r.Exists()).To(BeTrue())
					})
				})
			})

			Context("when applied to a scratchpad", func() {
				JustBeforeEach(func() {
					sp = repo.NewScratchpad("2019w28")
					err = tasks.GitClone().ApplyScratchpad(sp)
				})

				AfterEach(func() {
					os.RemoveAll(sp.Path())
				})

				It("Should not return an error", func() {
					Expect(err).ToNot(HaveOccurred())
				})

				It("should not log anything", func() {
					Expect(out.GetOperations()).To(BeEmpty())
				})

				It("Should not have created the scratchpad folder", func() {
					Expect(sp.Exists()).To(BeFalse())
				})
			})
		}

		Context("when using integrated cloning", func() {
			BeforeEach(func() {
				cfg.SetFeatures(&config.Features{
					NativeClone:   false,
					CreateRemote:  false,
					HttpTransport: true,
				})
			})

			runCloneTests()
		})

		Context("when using native cloning", func() {
			BeforeEach(func() {
				cfg.SetFeatures(&config.Features{
					NativeClone:   true,
					CreateRemote:  false,
					HttpTransport: true,
				})
			})

			runCloneTests()
		})

	})
})
