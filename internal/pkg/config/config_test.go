package config_test

import (
	"os"
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/test"

	. "github.com/smartystreets/goconvey/convey"
)

func TestConfig(t *testing.T) {
	Convey("Config", t, func() {
		Convey("Default()", func() {
			os.Setenv("DEV_DIRECTORY", test.GetTestPath())

			cfg := config.Default()

			Convey("Should return a default configuration", func() {
				So(cfg, ShouldNotBeNil)

				Convey("with a valid directory", func() {
					So(cfg.DevelopmentDirectory(), ShouldNotBeEmpty)
					So(cfg.DevelopmentDirectory(), ShouldEqual, test.GetTestPath())
				})

				Convey("with a github.com service entry", func() {
					So(cfg.GetServices(), ShouldNotBeEmpty)
					So(cfg.GetDefaultService(), ShouldNotBeNil)
					So(cfg.GetDefaultService().Domain(), ShouldEqual, "github.com")
				})

				Convey("with a shell command", func() {
					So(cfg.GetApps(), ShouldNotBeEmpty)
					So(cfg.GetDefaultApp(), ShouldNotBeNil)
					So(cfg.GetDefaultApp().Name(), ShouldEqual, "shell")
				})
			})
		})

		Convey("Load()", func() {
			Convey("When the config is missing", func() {
				cfg, err := config.Load(test.GetTestDataPath("config.missing.yml"))
				So(err, ShouldNotBeNil)
				So(err.Error(), ShouldStartWith, "config: unable to read config file")
				So(cfg, ShouldBeNil)
			})

			Convey("When the config is invalid", func() {
				cfg, err := config.Load(test.GetTestDataPath("config.invalid-yaml.yml"))
				So(err, ShouldNotBeNil)
				So(err.Error(), ShouldStartWith, "config: unable to parse config file")
				So(cfg, ShouldBeNil)
			})

			Convey("When the config is valid", func() {
				cfg, err := config.Load(test.GetTestDataPath("config.valid.yml"))
				So(err, ShouldBeNil)
				So(cfg, ShouldNotBeNil)
			})
		})

		Convey("GetServices()", func() {
			cfg := config.Default()

			Convey("Should return the list of registered services", func() {
				So(cfg.GetServices(), ShouldHaveLength, 1)
			})
		})

		Convey("GetDefaultService()", func() {
			cfg := config.Default()

			Convey("Should return the first defined service", func() {
				So(cfg.GetServices(), ShouldHaveLength, 1)
				So(cfg.GetDefaultService(), ShouldEqual, cfg.GetServices()[0])
			})
		})

		Convey("GetService(domain)", func() {
			cfg := config.Default()

			Convey("Should return nil if the service cannot be found", func() {
				So(cfg.GetService("notadomain.invalid"), ShouldBeNil)
			})

			Convey("Should return the default service if no domain is specified", func() {
				So(cfg.GetService(""), ShouldEqual, cfg.GetDefaultService())
			})

			Convey("Should return a service based on its domain name", func() {
				So(cfg.GetService("github.com"), ShouldNotBeNil)
				So(cfg.GetService("github.com").Domain(), ShouldEqual, "github.com")
			})
		})

		Convey("GetApps()", func() {
			cfg := config.Default()

			Convey("Should return the list of registered apps", func() {
				So(cfg.GetApps(), ShouldHaveLength, 1)
			})
		})

		Convey("GetDefaultApp()", func() {
			cfg := config.Default()

			Convey("Should return the first defined app", func() {
				So(cfg.GetApps(), ShouldHaveLength, 1)
				So(cfg.GetDefaultApp(), ShouldEqual, cfg.GetApps()[0])
			})
		})

		Convey("GetApp(name)", func() {
			cfg := config.Default()

			Convey("Should return nil if the app cannot be found", func() {
				So(cfg.GetApp("unknownapp"), ShouldBeNil)
			})

			Convey("Should return the default app if no name is specified", func() {
				So(cfg.GetApp(""), ShouldEqual, cfg.GetDefaultApp())
			})

			Convey("Should return a app based on its name", func() {
				So(cfg.GetApp("shell"), ShouldNotBeNil)
				So(cfg.GetApp("shell").Name(), ShouldEqual, "shell")
			})
		})

		Convey("GetAliases()", func() {
			cfg, err := config.Load(test.GetTestDataPath("config.valid.yml"))
			So(err, ShouldBeNil)
			So(cfg, ShouldNotBeNil)

			Convey("Should return the list of configured aliases", func() {
				So(cfg.GetAliases(), ShouldResemble, map[string]string{
					"gt": "github.com/SierraSoftworks/git-tool",
				})
			})
		})

		Convey("GetAlias(name)", func() {
			cfg, err := config.Load(test.GetTestDataPath("config.valid.yml"))
			So(err, ShouldBeNil)
			So(cfg, ShouldNotBeNil)

			Convey("Should return an empty string if the alias isn't found", func() {
				So(cfg.GetAlias("unknown"), ShouldBeEmpty)
			})

			Convey("Should return a configured alias", func() {
				So(cfg.GetAlias("gt"), ShouldEqual, "github.com/SierraSoftworks/git-tool")
			})
		})
	})
}
