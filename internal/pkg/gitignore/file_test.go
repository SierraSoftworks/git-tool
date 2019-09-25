package gitignore_test

import (
	"io/ioutil"
	"os"

	"github.com/SierraSoftworks/git-tool/internal/pkg/gitignore"
	"github.com/SierraSoftworks/git-tool/test"

	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
)

var _ = Describe("GitIgnore", func() {
	Describe("AddOrUpdate()", func() {
		var (
			filePath        string
			languages       []string
			originalContent string
			newContent      string
			err             error
		)

		BeforeEach(func() {
			filePath = test.GetTestDataPath("ignore", "oldgo.gitignore")
			languages = []string{}
			originalContent = ""
			newContent = ""
			err = nil
		})

		JustBeforeEach(func() {
			oc, ferr := ioutil.ReadFile(filePath)
			if ferr == nil {
				originalContent = string(oc)
			}

			err = gitignore.AddOrUpdate(filePath, languages...)

			oc, ferr = ioutil.ReadFile(filePath)
			if ferr == nil {
				newContent = string(oc)
			}
		})

		AfterEach(func() {
			ioutil.WriteFile(filePath, []byte(originalContent), os.ModePerm)
		})

		Context("With a file which doesn't exist", func() {
			BeforeEach(func() {
				filePath = test.GetTestDataPath("ignore", ".gitignore")
			})

			AfterEach(func() {
				os.RemoveAll(filePath)
			})

			Context("With no languages provided", func() {
				It("Should not report an error", func() {
					Expect(err).To(BeNil())
				})

				It("Should not start with any content", func() {
					Expect(originalContent).To(BeEmpty())
				})

				It("Should not write any content", func() {
					Expect(newContent).To(BeEmpty())
				})
			})

			Context("With a language provided", func() {
				BeforeEach(func() {
					languages = []string{"go"}
				})

				It("Should not report an error", func() {
					Expect(err).To(BeNil())
				})

				It("Should not start with any content", func() {
					Expect(originalContent).To(BeEmpty())
				})

				It("Should write the ignore content", func() {
					Expect(newContent).ToNot(BeEmpty())
				})
			})
		})

		Context("With an old file", func() {
			BeforeEach(func() {
				filePath = test.GetTestDataPath("ignore", "oldgo.gitignore")
			})

			Context("With no languages provided", func() {
				It("Should not report an error", func() {
					Expect(err).To(BeNil())
				})

				It("Should start with some content", func() {
					Expect(originalContent).ToNot(BeEmpty())
				})

				It("Should write some new content", func() {
					Expect(newContent).ToNot(BeEmpty())
					Expect(newContent).ToNot(Equal(originalContent))
				})
			})

			Context("With the same language provided", func() {
				BeforeEach(func() {
					languages = []string{"go"}
				})

				It("Should not report an error", func() {
					Expect(err).To(BeNil())
				})
			})
		})
	})
})
