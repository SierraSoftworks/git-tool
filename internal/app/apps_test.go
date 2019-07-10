package app

import (
	"testing"
	. "github.com/smartystreets/goconvey/convey"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
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

				So(out.GetLines(), ShouldNotBeEmpty)
				So(out.GetLines(), ShouldHaveLength, len(di.GetConfig().GetApps()))
			})

			Convey("Should appear in the root completions list", func() {
				defer out.Reset()

				So(app.Run([]string{
					"gt",
					"complete",
					"gt ",
				}), ShouldBeNil)

				So(out.GetLines(), ShouldContain, "apps")
			})

			Convey("Should return an empty completions list", func() {
				defer out.Reset()

				So(app.Run([]string{
					"gt",
					"complete",
					"--position=8",
					"gt apps",
				}), ShouldBeNil)

				So(out.GetLines(), ShouldBeEmpty)
			})
		})
	})
}