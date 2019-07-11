package gitignore_test

import (
	"github.com/SierraSoftworks/git-tool/internal/pkg/gitignore"
	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
)

var _ = Describe("GitIgnore", func() {
	Describe("List()", func() {
		var (
			list    []string
			listErr error
		)

		BeforeEach(func() {
			list, listErr = gitignore.List()
		})

		It("Should not return an error", func() {
			Expect(listErr).ToNot(HaveOccurred())
		})

		It("Should return at least one item", func() {
			Expect(len(list)).To(BeNumerically(">", 0))
		})

		It("Should split the items in the list correctly", func() {
			for _, item := range list {
				Expect(item).ToNot(BeEmpty())
				Expect(item).ToNot(ContainSubstring(","))
				Expect(item).ToNot(ContainSubstring("\n"))
			}
		})
	})

	Describe("Ignore()", func() {
		var (
			langs  []string
			ignore string
			err    error
		)

		BeforeEach(func() {
			langs = []string{}
		})

		JustBeforeEach(func() {
			ignore, err = gitignore.Ignore(langs...)
		})

		Context("With an unrecognized language", func() {
			BeforeEach(func() {
				langs = []string{"thisisnotareallanguage"}
			})

			It("Should not return an error", func() {
				Expect(err).To(BeNil())
			})

			It("Should return an emppty ignore file", func() {
				Expect(ignore).To(BeEmpty())
			})
		})

		Context("With a single language", func() {
			BeforeEach(func() {
				langs = []string{"go"}
			})

			It("Should not return an error", func() {
				Expect(err).To(BeNil())
			})

			It("Should return a valid ignore file", func() {
				Expect(ignore).ToNot(BeEmpty())
				Expect(ignore).To(ContainSubstring(".exe~"))
			})
		})

		Context("With multiple languages", func() {
			BeforeEach(func() {
				langs = []string{"go", "node"}
			})

			It("Should not return an error", func() {
				Expect(err).To(BeNil())
			})

			It("Should return a valid ignore file", func() {
				Expect(ignore).ToNot(BeEmpty())
				Expect(ignore).To(ContainSubstring(".exe~"))
				Expect(ignore).To(ContainSubstring("node_modules"))
			})
		})
	})
})
