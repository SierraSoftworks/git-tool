package app

import (
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	. "github.com/smartystreets/goconvey/convey"
)

func TestAppsSubcommand(t *testing.T) {
	Convey("Apps", t, func() {
		app := NewApp()

		out := &di.TestOutput{}
		di.SetOutput(out)

		Convey("gt apps", func() {
			Convey("Should be registered", func() {
				cmd := app.Command("apps")
				So(cmd, ShouldNotBeNil)
			})

			Convey("Should print out the list of apps which have been configured", func() {
				defer out.Reset()

				So(app.Run([]string{
					"gt",
					"apps",
				}), ShouldBeNil)

				So(out.GetOperations(), ShouldNotBeEmpty)
				So(out.GetOperations(), ShouldHaveLength, len(di.GetConfig().GetApps()))
			})

			Convey("Should appear in the root completions list", func() {
				defer out.Reset()

				So(app.Run([]string{
					"gt",
					"complete",
					"gt",
				}), ShouldBeNil)

				So(out.GetOperations(), ShouldContain, "apps\n")
			})

			Convey("Should return an empty completions list", func() {
				defer out.Reset()

				So(app.Run([]string{
					"gt",
					"complete",
					"--position=8",
					"gt apps",
				}), ShouldBeNil)

				So(out.GetOperations(), ShouldBeEmpty)
			})
		})
	})
}
