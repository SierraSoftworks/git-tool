package config_test

import (
	"path/filepath"
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/test"
	testmodels "github.com/SierraSoftworks/git-tool/test/models"

	. "github.com/smartystreets/goconvey/convey"
)

func TestService(t *testing.T) {
	Convey("Service", t, func() {
		cfg, err := config.Load(test.GetTestDataPath("config.valid.yml"))
		So(err, ShouldBeNil)
		So(cfg, ShouldNotBeNil)

		Convey("should return the correct domain when requested", func() {
			svc := cfg.GetService("gitlab.com")
			So(svc, ShouldNotBeNil)
			So(svc.Domain(), ShouldEqual, "gitlab.com")
		})

		Convey("should return the correct directory glob when requested", func() {
			svc := cfg.GetService("dev.azure.com")
			So(svc, ShouldNotBeNil)
			So(svc.DirectoryGlob(), ShouldEqual, "*/*/*")
		})

		Convey("for github.com", func() {
			svc := cfg.GetService("github.com")

			Convey("should render the website address correctly", func() {
				r := &testmodels.TestRepo{
					ModelFullName: "sierrasoftworks/test1",
					ModelService:  svc,
					ModelPath:     filepath.Join(test.GetTestPath(), svc.Domain(), "sierrasoftworks", "test1"),
					ModelValid:    false,
					ModelExists:   false,
				}

				So(svc.Website(r), ShouldEqual, "https://github.com/sierrasoftworks/test1")
			})

			Convey("should render the git URL address correctly", func() {
				r := &testmodels.TestRepo{
					ModelFullName: "sierrasoftworks/test1",
					ModelService:  svc,
					ModelPath:     filepath.Join(test.GetTestPath(), svc.Domain(), "sierrasoftworks", "test1"),
					ModelValid:    false,
					ModelExists:   false,
				}

				So(svc.GitURL(r), ShouldEqual, "git@github.com:sierrasoftworks/test1.git")
			})

			Convey("should render the git HTTP URL address correctly", func() {
				r := &testmodels.TestRepo{
					ModelFullName: "sierrasoftworks/test1",
					ModelService:  svc,
					ModelPath:     filepath.Join(test.GetTestPath(), svc.Domain(), "sierrasoftworks", "test1"),
					ModelValid:    false,
					ModelExists:   false,
				}

				So(svc.HttpURL(r), ShouldEqual, "https://github.com/sierrasoftworks/test1.git")
			})
		})

	})
}
