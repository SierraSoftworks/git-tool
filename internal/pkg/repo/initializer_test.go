package repo_test

import (
	"os"
	"path/filepath"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/mocks"
	"github.com/SierraSoftworks/git-tool/internal/pkg/repo"
	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/SierraSoftworks/git-tool/test"
	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
)

var _ = Describe("Initializer", func() {
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
	})

	AfterEach(func() {
		os.Chdir(test.GetProjectRoot())
	})

	Describe("CreateScratchpad()", func() {
		Context("when the directory exists", func() {
			BeforeEach(func() {
				sp = repo.NewScratchpad("2019w15")
				err = di.GetInitializer().CreateScratchpad(sp)
			})

			It("should not return an error", func() {
				Expect(err).ToNot(HaveOccurred())
			})

			It("should not log anything", func() {
				Expect(out.GetOperations()).To(BeEmpty())
			})

			It("should leave the directory in place", func() {
				Expect(sp.Exists()).To(BeTrue())
			})
		})

		Context("when the directory doesn't exist", func() {
			BeforeEach(func() {
				sp = repo.NewScratchpad("2019w01")
				err = di.GetInitializer().CreateScratchpad(sp)
			})

			AfterEach(func() {
				os.RemoveAll(sp.Path())
			})

			It("should not return an error", func() {
				Expect(err).ToNot(HaveOccurred())
			})

			It("should not log anything", func() {
				Expect(out.GetOperations()).To(BeEmpty())
			})

			It("should create the directory", func() {
				s := repo.NewScratchpad("2019w15")
				Expect(s.Exists()).To(BeTrue())
			})
		})
	})

	Describe("Init()", func() {
		BeforeEach(func() {
			cfg.SetFeatures(&config.Features{
				NativeClone:  false,
				CreateRemote: false,
			})
		})

		Context("when the repo doesn't exist", func() {
			BeforeEach(func() {
				r = repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test3")
				err = di.GetInitializer().Init(r)
			})

			AfterEach(func() {
				os.RemoveAll(r.Path())
			})

			It("should not return an error", func() {
				Expect(err).ToNot(HaveOccurred())
			})

			It("should not log anything", func() {
				Expect(out.GetOperations()).To(BeEmpty())
			})

			It("should have created the repo", func() {
				Expect(r.Exists()).To(BeTrue())
			})

			It("should have initialized the repo", func() {
				Expect(r.Valid()).To(BeTrue())
			})
		})

		Context("when the repo exists", func() {
			BeforeEach(func() {
				r = repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test2")
				err = di.GetInitializer().Init(r)
			})

			AfterEach(func() {
				os.RemoveAll(filepath.Join(r.Path(), ".git"))
			})

			It("should not return an error", func() {
				Expect(err).ToNot(HaveOccurred())
			})

			It("should not log anything", func() {
				Expect(out.GetOperations()).To(BeEmpty())
			})

			It("should have created the repo", func() {
				Expect(r.Exists()).To(BeTrue())
			})

			It("should have initialized the repo", func() {
				Expect(r.Valid()).To(BeTrue())
			})
		})
	})

	Describe("Clone()", func() {
		Context("when using integrated cloning", func() {
			BeforeEach(func() {
				cfg.SetFeatures(&config.Features{
					NativeClone:   false,
					CreateRemote:  false,
					HttpTransport: true,
				})
			})

			Context("when the repo already exists", func() {
				BeforeEach(func() {
					r = repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test1")
					err = di.GetInitializer().Clone(r)
				})

				AfterEach(func() {
					os.RemoveAll(filepath.Join(r.Path(), ".git"))
				})

				It("should not return an error", func() {
					Expect(err).ToNot(HaveOccurred())
				})

				It("should not log anything", func() {
					Expect(out.GetOperations()).To(BeEmpty())
				})

				It("should have created the repo", func() {
					Expect(r.Exists()).To(BeTrue())
				})

				It("should not have modified the repo", func() {
					Expect(r.Valid()).To(BeFalse())
				})
			})

			Context("when the repo doesn't exist", func() {
				BeforeEach(func() {
					r = repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/git-tool")
					err = di.GetInitializer().Clone(r)
				})

				AfterEach(func() {
					os.RemoveAll(r.Path())
				})

				It("should not return an error", func() {
					Expect(err).ToNot(HaveOccurred())
				})

				It("should not log anything", func() {
					Expect(out.GetOperations()).To(BeEmpty())
				})

				It("should have created the repo", func() {
					Expect(r.Exists()).To(BeTrue())
				})

				It("should have a valid repo", func() {
					Expect(r.Valid()).To(BeTrue())
				})
			})
		})

		Context("when using native cloning", func() {
			BeforeEach(func() {
				cfg.SetFeatures(&config.Features{
					NativeClone:   true,
					CreateRemote:  false,
					HttpTransport: true,
				})

				di.SetLauncher(di.DefaultLauncher())
			})

			Context("when the repo already exists", func() {
				BeforeEach(func() {
					r = repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test1")
					err = di.GetInitializer().Clone(r)
				})

				AfterEach(func() {
					os.RemoveAll(filepath.Join(r.Path(), ".git"))
				})

				It("should not return an error", func() {
					Expect(err).ToNot(HaveOccurred())
				})

				It("should not log anything", func() {
					Expect(out.GetOperations()).To(BeEmpty())
				})

				It("should have created the repo", func() {
					Expect(r.Exists()).To(BeTrue())
				})

				It("should not have modified the repo", func() {
					Expect(r.Valid()).To(BeFalse())
				})
			})

			Context("when the repo doesn't exist", func() {
				BeforeEach(func() {
					r = repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/git-tool")
					err = di.GetInitializer().Clone(r)
				})

				AfterEach(func() {
					os.RemoveAll(r.Path())
				})

				It("should not return an error", func() {
					Expect(err).ToNot(HaveOccurred())
				})

				It("should log the Git output", func() {
					Expect(out.GetOperations()).ToNot(BeEmpty())
				})

				It("should have created the repo", func() {
					Expect(r.Exists()).To(BeTrue())
				})

				It("should have a valid repo", func() {
					Expect(r.Valid()).To(BeTrue())
				})
			})
		})
	})
})
