package gitignore_test

import (
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/gitignore"
	. "github.com/smartystreets/goconvey/convey"
)

func TestGitignore(t *testing.T) {
	Convey("Gitignore", t, func() {
		Convey("List()", func() {
			list, err := gitignore.List()
			So(err, ShouldBeNil)
			So(list, ShouldNotBeNil)

			Convey("Should return at least one item", func() {
				So(len(list), ShouldBeGreaterThan, 0)
			})

			Convey("Should split the items in the list correctly", func() {
				for _, item := range list {
					So(item, ShouldNotContainSubstring, ",")
					So(item, ShouldNotContainSubstring, "\n")
					So(item, ShouldNotBeEmpty)
				}
			})
		})

		Convey("Ignore()", func() {
			Convey("with no languages", func() {
				ignore, err := gitignore.Ignore()
				So(err, ShouldBeNil)
				So(ignore, ShouldBeEmpty)
			})

			Convey("with an unrecognized language", func() {
				ignore, err := gitignore.Ignore("thisisnotareallanguage")
				So(err, ShouldBeNil)
				So(ignore, ShouldBeEmpty)
			})

			Convey("with a single language", func() {
				ignore, err := gitignore.Ignore("csharp")
				So(err, ShouldBeNil)
				So(ignore, ShouldNotBeEmpty)
				So(ignore, ShouldContainSubstring, "csharp")
			})

			Convey("with multiple languages", func() {
				ignore, err := gitignore.Ignore("csharp", "node")
				So(err, ShouldBeNil)
				So(ignore, ShouldNotBeEmpty)
				So(ignore, ShouldContainSubstring, "csharp")
				So(ignore, ShouldContainSubstring, "node_modules")
			})
		})
	})
}
