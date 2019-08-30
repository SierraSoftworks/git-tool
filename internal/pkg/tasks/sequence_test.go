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

var _ = Describe("Sequenced Tasks", func() {
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

		r = repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test1")
		sp = repo.NewScratchpad("2019w15")
	})

	AfterEach(func() {
		os.Chdir(test.GetProjectRoot())
	})

	Describe("Sequence()", func() {
		Context("When called with no tasks", func() {
			It("Should return a task", func() {
				Expect(tasks.Sequence()).ToNot(BeNil())
			})

			Context("when applied to a repo", func() {
				BeforeEach(func() {
					err = tasks.Sequence().ApplyRepo(r)
				})

				It("Should not return an error", func() {
					Expect(err).ToNot(HaveOccurred())
				})
			})

			Context("when applied to a scratchpad", func() {
				BeforeEach(func() {
					err = tasks.Sequence().ApplyScratchpad(sp)
				})

				It("Should not return an error", func() {
					Expect(err).ToNot(HaveOccurred())
				})
			})
		})

		Context("When called with a sequence of tasks", func() {
			var (
				firstCalled  = false
				secondCalled = false
			)

			BeforeEach(func() {
				err = nil
				firstCalled = false
				secondCalled = false
			})

			Context("when applied to a repo", func() {
				BeforeEach(func() {
					t := tasks.Sequence(&TestTask{
						OnRepo: func(rr models.Repo) error {
							firstCalled = true
							Expect(secondCalled).To(BeFalse())
							Expect(rr).To(Equal(r))
							return nil
						},
					}, &TestTask{
						OnRepo: func(rr models.Repo) error {
							secondCalled = true
							Expect(firstCalled).To(BeTrue())
							Expect(rr).To(Equal(r))
							return nil
						},
					})
					
					err = t.ApplyRepo(r)
				})

				It("Should not return an error", func() {
					Expect(err).ToNot(HaveOccurred())
				})

				It("Should have called the first task", func() {
					Expect(firstCalled).To(BeTrue())
				})

				It("Should have called the second task", func() {
					Expect(secondCalled).To(BeTrue())
				})
			})

			Context("when applied to a scratchpad", func() {
				BeforeEach(func() {
					t := tasks.Sequence(&TestTask{
						OnScratchpad: func(ss models.Scratchpad) error {
							firstCalled = true
							Expect(secondCalled).To(BeFalse())
							Expect(ss).To(Equal(sp))
							return nil
						},
					}, &TestTask{
						OnScratchpad: func(ss models.Scratchpad) error {
							secondCalled = true
							Expect(firstCalled).To(BeTrue())
							Expect(ss).To(Equal(sp))
							return nil
						},
					})

					err = t.ApplyScratchpad(sp)
				})

				It("Should not return an error", func() {
					Expect(err).ToNot(HaveOccurred())
				})

				It("Should have called the first task", func() {
					Expect(firstCalled).To(BeTrue())
				})

				It("Should have called the second task", func() {
					Expect(secondCalled).To(BeTrue())
				})
			})
		})
	})
})
