package tasks_test

import (
	"os"
	"path/filepath"

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

var _ = Describe("Git Init Task", func() {
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

	Describe("GitInit()", func() {
		Context("when applied to a repo", func() {
			JustBeforeEach(func() {
				err = tasks.GitInit().ApplyRepo(r)
			})

			Context("which doesn't exist", func() {
				BeforeEach(func() {
					r = repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test3")
				})

				AfterEach(func() {
					os.RemoveAll(r.Path())
				})

				It("Should not return an error", func() {
					Expect(err).ToNot(HaveOccurred())
				})

				It("should not log anything", func() {
					Expect(out.GetOperations()).To(BeEmpty())
				})

				It("Should have created the repository folder", func() {
					Expect(r.Exists()).To(BeTrue())
				})

				It("Should have initialized the repository", func() {
					Expect(r.Valid()).To(BeTrue())
				})
			})
			
			Context("which does exist", func() {
				BeforeEach(func() {
					r = repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test2")
				})

				AfterEach(func() {
					os.RemoveAll(filepath.Join(r.Path(), ".git"))
				})

				It("Should not return an error", func() {
					Expect(err).ToNot(HaveOccurred())
				})

				It("should not log anything", func() {
					Expect(out.GetOperations()).To(BeEmpty())
				})

				It("Should have created the repository folder", func() {
					Expect(r.Exists()).To(BeTrue())
				})

				It("Should have initialized the repository", func() {
					Expect(r.Valid()).To(BeTrue())
				})
			})
		})

		Context("when applied to a scratchpad", func() {
			JustBeforeEach(func() {
				err = tasks.GitInit().ApplyScratchpad(sp)
			})

			Context("which doesn't exist", func() {
				BeforeEach(func() {
					sp = repo.NewScratchpad("2019w28")
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
			
			Context("which does exist", func() {
				BeforeEach(func() {
					sp = repo.NewScratchpad("2019w27")
				})

				It("Should not return an error", func() {
					Expect(err).ToNot(HaveOccurred())
				})

				It("should not log anything", func() {
					Expect(out.GetOperations()).To(BeEmpty())
				})

				It("Should have left the scratchpad folder in place", func() {
					Expect(sp.Exists()).To(BeTrue())
				})
			})
		})
	})
})