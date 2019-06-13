package config_test

import (
	"os"
	"path/filepath"
	"testing"

	testmodels "github.com/SierraSoftworks/git-tool/test/models"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/test"

	. "github.com/smartystreets/goconvey/convey"
)

func TestApp(t *testing.T) {
	Convey("App", t, func() {
		cfg, err := config.Load(test.GetTestDataPath("config.valid.yml"))
		So(err, ShouldBeNil)
		So(cfg, ShouldNotBeNil)

		Convey("should return the correct name when requested", func() {
			app := cfg.GetApp("make")
			So(app, ShouldNotBeNil)
			So(app.Name(), ShouldEqual, "make")
		})

		Convey("should return a command to launch the app in the context of a repo", func() {
			app := cfg.GetApp("make")
			So(app, ShouldNotBeNil)

			r := &testmodels.TestRepo{
				ModelFullName: "sierrasoftworks/test1",
				ModelService:  cfg.GetService("github.com"),
				ModelPath:     filepath.Join(test.GetTestPath(), "github.com", "sierrasoftworks", "test1"),
				ModelValid:    false,
				ModelExists:   false,
			}

			cmd, err := app.GetCmd(r)
			So(err, ShouldBeNil)
			So(cmd, ShouldNotBeNil)
			So(cmd.Dir, ShouldEqual, r.Path())
			So(cmd.Path, ShouldEqual, "make")
			So(cmd.Args, ShouldResemble, []string{"make", "build"})
			So(cmd.Env, ShouldResemble, append(os.Environ(), "CI_SERVER=0", "REPO=sierrasoftworks/test1", "GITHOST=github.com"))
		})
	})
}
