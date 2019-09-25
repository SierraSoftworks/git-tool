package gitignore_test

import (
	"strings"

	"github.com/SierraSoftworks/git-tool/internal/pkg/gitignore"

	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
)

var _ = Describe("FileHeader", func() {
	Describe("Parse()", func() {
		var (
			header  *gitignore.ManagedFileSection
			content string
		)

		BeforeEach(func() {
			content = ""
		})

		JustBeforeEach(func() {
			header = gitignore.ParseSection(content)
		})

		Describe("When it is at the start of the file", func() {
			BeforeEach(func() {
				content = strings.TrimSpace(`
## -------- Managed by Git Tool -------- ##
## Add any custom rules above this block ##
## ------------------------------------- ##
## @languages: go,rust, csharp
*.exe`)
			})

			It("should return the header", func() {
				Expect(header).ToNot(BeNil())
			})

			It("should have the right languages", func() {
				Expect(header).ToNot(BeNil())
				Expect(header.Languages).To(BeEquivalentTo([]string{"go", "rust", "csharp"}))
			})

			It("should have the right prologue", func() {
				Expect(header).ToNot(BeNil())
				Expect(header.Prologue).To(Equal(""))
			})

			It("should have the right content", func() {
				Expect(header).ToNot(BeNil())
				Expect(header.Content).To(Equal("*.exe"))
			})
		})

		Describe("When it is at the end of the file", func() {
			BeforeEach(func() {
				content = strings.TrimSpace(`
junit.xml
bin/

## -------- Managed by Git Tool -------- ##
## Add any custom rules above this block ##
## ------------------------------------- ##
## @languages: csharp, java
*.exe
*.obj`)
			})

			It("should return the header", func() {
				Expect(header).ToNot(BeNil())
			})

			It("should have the right languages", func() {
				Expect(header).ToNot(BeNil())
				Expect(header.Languages).To(BeEquivalentTo([]string{"csharp", "java"}))
			})

			It("should have the right prologue", func() {
				Expect(header).ToNot(BeNil())
				Expect(header.Prologue).To(Equal("junit.xml\nbin/\n"))
			})

			It("should have the right content", func() {
				Expect(header).ToNot(BeNil())
				Expect(header.Content).To(Equal("*.exe\n*.obj"))
			})
		})
	})
})
