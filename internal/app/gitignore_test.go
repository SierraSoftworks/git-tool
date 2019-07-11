package app

import (
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	. "github.com/smartystreets/goconvey/convey"
)

func TestGitIgnoreSubcommand(t *testing.T) {
	Convey("GitIgnore", t, func() {
		app := NewApp()

		out := &di.TestOutput{}
		di.SetOutput(out)

		Convey("gt ignore", func() {
			Convey("Should be registered", func() {
				cmd := app.Command("ignore")
				So(cmd, ShouldNotBeNil)
			})

			Convey("Should print out the list of languages when no arguments are provided", func() {
				defer out.Reset()

				So(app.Run([]string{
					"gt",
					"ignore",
				}), ShouldBeNil)

				So(out.GetOperations(), ShouldNotBeEmpty)
				So(out.GetOperations(), ShouldContain, " - csharp\n")
				So(out.GetOperations(), ShouldContain, " - go\n")
			})

			Convey("Should print out the ignore file when languages are provided", func() {
				defer out.Reset()

				So(app.Run([]string{
					"gt",
					"ignore",
					"go",
				}), ShouldBeNil)

				So(out.GetOperations(), ShouldNotBeEmpty)
				So(out.GetOperations(), ShouldHaveLength, 1)
				So(out.GetOperations()[0], ShouldContainSubstring, "*.exe~")
			})

			Convey("Should appear in the root completions list", func() {
				defer out.Reset()

				So(app.Run([]string{
					"gt",
					"complete",
					"gt",
				}), ShouldBeNil)

				So(out.GetOperations(), ShouldContain, "ignore\n")
			})

			Convey("Should return a completion list with the list of valid languages", func() {
				defer out.Reset()

				So(app.Run([]string{
					"gt",
					"complete",
					"gt ignore ",
				}), ShouldBeNil)

				So(out.GetOperations(), ShouldNotBeEmpty)
				So(out.GetOperations(), ShouldContain, "csharp\n")
				So(out.GetOperations(), ShouldContain, "go\n")
			})
		})
	})
}
