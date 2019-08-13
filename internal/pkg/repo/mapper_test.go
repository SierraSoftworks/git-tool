package repo_test

import (
	"os"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/mocks"
	"github.com/SierraSoftworks/git-tool/internal/pkg/repo"
	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/SierraSoftworks/git-tool/test"
	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
)

var _ = Describe("Mapper", func() {
	var (
		out    *mocks.Output
		launch *mocks.Launcher
		init   *mocks.Initializer
		r      models.Repo
		cfg    *mocks.Config
		err    error
	)

	BeforeEach(func() {
		cfg = mocks.NewConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))
		out = &mocks.Output{}
		launch = &mocks.Launcher{}
		init = &mocks.Initializer{}
		di.SetOutput(out)
		di.SetLauncher(launch)
		di.SetInitializer(init)
		di.SetMapper(&repo.Mapper{})
	})

	JustBeforeEach(func() {
		di.SetConfig(cfg)
	})

	AfterEach(func() {
		os.Chdir(test.GetProjectRoot())
	})

	Describe("GetBestRepo()", func() {
		Context("When an alias is provided", func() {
			BeforeEach(func() {
				cfg.AddAlias("alias1", "github.com/sierrasoftworks/test1")
				cfg.AddAlias("alias2", "nonexistent.com/namespace/repo")
			})

			Context("which matches a known repo", func() {
				JustBeforeEach(func() {
					r, err = di.GetMapper().GetBestRepo("alias1")
				})

				It("should not return an error", func() {
					Expect(err).ToNot(HaveOccurred())
				})

				It("should return the correct repo", func() {
					Expect(r).ToNot(BeNil())
					Expect(r.Service().Domain()).To(Equal("github.com"))
					Expect(r.FullName()).To(Equal("sierrasoftworks/test1"))
				})
			})

			Context("which doesn't match a known service", func() {
				JustBeforeEach(func() {
					r, err = di.GetMapper().GetBestRepo("alias2")
				})

				It("should not return an error", func() {
					Expect(err).ToNot(HaveOccurred())
				})

				It("should not return a repo", func() {
					Expect(r).To(BeNil())
				})
			})

			Context("which hasn't been configured", func() {
				JustBeforeEach(func() {
					r, err = di.GetMapper().GetBestRepo("alias3")
				})

				It("should not return an error", func() {
					Expect(err).ToNot(HaveOccurred())
				})

				It("should not return a repo", func() {
					Expect(r).To(BeNil())
				})
			})
		})

		Context("When a full repo name is provided", func() {
			Context("with a service included", func() {
				Context("which doesn't exist", func() {
					JustBeforeEach(func() {
						r, err = di.GetMapper().GetBestRepo("nonexistent.com/namespace/repo")
					})

					It("should not return an error", func() {
						Expect(err).ToNot(HaveOccurred())
					})

					It("should not return a repo", func() {
						Expect(r).To(BeNil())
					})
				})

				Context("which does exist", func() {
					JustBeforeEach(func() {
						r, err = di.GetMapper().GetBestRepo("github.com/sierrasoftworks/test1")
					})

					It("should not return an error", func() {
						Expect(err).ToNot(HaveOccurred())
					})

					It("should return the correct repo", func() {
						Expect(r).ToNot(BeNil())
						Expect(r.Service().Domain()).To(Equal("github.com"))
						Expect(r.FullName()).To(Equal("sierrasoftworks/test1"))
					})
				})
			})

			Context("without a service included", func() {
				JustBeforeEach(func() {
					r, err = di.GetMapper().GetBestRepo("sierrasoftworks/test1")
				})

				It("should not return an error", func() {
					Expect(err).ToNot(HaveOccurred())
				})

				It("should return the correct repo", func() {
					Expect(r).ToNot(BeNil())
					Expect(r.Service().Domain()).To(Equal("github.com"))
					Expect(r.FullName()).To(Equal("sierrasoftworks/test1"))
				})
			})
		})

		Context("When a partial repo name is provided", func() {
			Context("which matches a single repo", func() {
				JustBeforeEach(func() {
					r, err = di.GetMapper().GetBestRepo("ghsstst1")
				})

				It("should not return an error", func() {
					Expect(err).ToNot(HaveOccurred())
				})

				It("should return the correct repo", func() {
					Expect(r).ToNot(BeNil())
					Expect(r.Service().Domain()).To(Equal("github.com"))
					Expect(r.FullName()).To(Equal("sierrasoftworks/test1"))
				})
			})

			Context("which matches more than one repo", func() {
				JustBeforeEach(func() {
					r, err = di.GetMapper().GetBestRepo("test1")
				})

				It("should not return an error", func() {
					Expect(err).ToNot(HaveOccurred())
				})

				It("should not return a repo", func() {
					Expect(r).To(BeNil())
				})
			})

			Context("which doesn't match a repo", func() {
				JustBeforeEach(func() {
					r, err = di.GetMapper().GetBestRepo("unrecognized")
				})

				It("should not return an error", func() {
					Expect(err).ToNot(HaveOccurred())
				})

				It("should not return a repo", func() {
					Expect(r).To(BeNil())
				})
			})
		})
	})
})
