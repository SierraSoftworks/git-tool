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

var _ = Describe("Git Checkout Task", func() {
	var (
		ref string
		out *mocks.Output
		r   models.Repo
		cfg *mocks.Config
		err error
	)

	BeforeEach(func() {
		ref = "master"
		cfg = mocks.NewConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))
		out = &mocks.Output{}
		di.SetOutput(out)
		di.SetLauncher(di.DefaultLauncher())
		di.SetMapper(&repo.Mapper{})
		di.SetInitializer(&repo.Initializer{})
		di.SetConfig(cfg)

		r = repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test1")
	})

	AfterEach(func() {
		os.Chdir(test.GetProjectRoot())
	})

	Describe("GitCheckout()", func() {
		Context("when applied to a repo", func() {
			JustBeforeEach(func() {
				err = tasks.GitCheckout(ref, false).ApplyRepo(r)
			})

			Context("which doesn't exist locally", func() {
				BeforeEach(func() {
					r = repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test3")
				})

				AfterEach(func() {
					os.RemoveAll(r.Path())
				})

				It("should not log anything", func() {
					Expect(out.GetOperations()).To(BeEmpty())
				})

				It("Should return an error", func() {
					Expect(err).To(HaveOccurred())
				})

				It("Should not have created the repo folder", func() {
					Expect(r.Exists()).To(BeFalse())
				})
			})

			Context("which exists locally but is not initialized", func() {
				BeforeEach(func() {
					r = repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test1")
				})

				It("should not log anything", func() {
					Expect(out.GetOperations()).To(BeEmpty())
				})

				It("Should return an error", func() {
					Expect(err).To(HaveOccurred())
				})

				It("Should still have the local repo folder", func() {
					Expect(r.Exists()).To(BeTrue())
				})
			})

			Context("which exists locally and is initialized", func() {
				var cloneError error

				BeforeEach(func() {
					r = repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test4")

					cloneError = tasks.Sequence(
						tasks.NewFolder(),
						tasks.GitInit(),
						tasks.NewFile("README.md", []byte("# Test Repo")),
						tasks.GitCommit("Initial Commit", "README.md"),
						tasks.GitCheckout("master", false),
					).ApplyRepo(r)
				})

				AfterEach(func() {
					os.RemoveAll(r.Path())
				})

				It("Should not fail to clone", func() {
					Expect(cloneError).ToNot(HaveOccurred())
				})

				Context("With a branch which exists", func() {
					BeforeEach(func() {
						ref = "master"
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

					It("Should still have the branch", func() {
						branches, err := di.GetMapper().GetBranches(r)
						Expect(err).ToNot(HaveOccurred())
						Expect(branches).To(ContainElement(ref))
					})
				})

				Context("With a branch which doesn't exist", func() {
					BeforeEach(func() {
						ref = "test"
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

					It("Should create the branch", func() {
						branches, err := di.GetMapper().GetBranches(r)
						Expect(err).ToNot(HaveOccurred())
						Expect(branches).To(ContainElement(ref))
					})
				})
			})
		})

		Context("when applied to a scratchpad", func() {
			var sp models.Scratchpad

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
	})
})
