package config_test

import (
	"os"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/test"

	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
)

var _ = Describe("Config", func() {
	BeforeSuite(func() {
		os.Setenv("DEV_DIRECTORY", test.GetTestPath())
	})

	Describe("Default()", func() {
		It("Should return a config", func() {
			Expect(config.Default()).ToNot(BeNil())
		})

		It("Should use the right development directory", func() {
			Expect(config.Default().DevelopmentDirectory()).To(Equal(test.GetTestPath()))
		})

		It("Should have a service entry for GitHub", func() {
			Expect(config.Default().GetServices()).ToNot(BeEmpty())
			Expect(config.Default().GetDefaultService()).ToNot(BeNil())
		})

		It("Should have an app entry for the shell", func() {
			Expect(config.Default().GetApps()).ToNot(BeEmpty())
			Expect(config.Default().GetDefaultApp()).ToNot(BeNil())
			Expect(config.Default().GetDefaultApp().Name()).To(Equal("shell"))
			Expect(config.Default().GetApp("shell")).ToNot(BeNil())
		})
	})

	Describe("DefaultForDirectory(dir)", func() {
		It("Should return a config", func() {
			Expect(config.DefaultForDirectory(test.GetTestPath())).ToNot(BeNil())
		})

		It("Should use the right development directory", func() {
			Expect(config.DefaultForDirectory(test.GetTestPath()).DevelopmentDirectory()).To(Equal(test.GetTestPath()))
		})

		It("Should have a service entry for GitHub", func() {
			Expect(config.DefaultForDirectory(test.GetTestPath()).GetServices()).ToNot(BeEmpty())
			Expect(config.DefaultForDirectory(test.GetTestPath()).GetDefaultService()).ToNot(BeNil())
		})

		It("Should have an app entry for the shell", func() {
			Expect(config.DefaultForDirectory(test.GetTestPath()).GetApps()).ToNot(BeEmpty())
			Expect(config.DefaultForDirectory(test.GetTestPath()).GetDefaultApp()).ToNot(BeNil())
			Expect(config.DefaultForDirectory(test.GetTestPath()).GetDefaultApp().Name()).To(Equal("shell"))
			Expect(config.DefaultForDirectory(test.GetTestPath()).GetApp("shell")).ToNot(BeNil())
		})
	})

	Describe("Load()", func() {
		var (
			cfg     config.Config
			cfgFile string
			cfgErr  error
		)

		BeforeEach(func() {
			cfgFile = "config.valid.yml"
		})

		JustBeforeEach(func() {
			cfg, cfgErr = config.Load(test.GetTestDataPath(cfgFile))
		})

		Context("With a missing file name", func() {
			BeforeEach(func() {
				cfgFile = "config.missing.yml"
			})

			It("Should return an error", func() {
				Expect(cfgErr).To(HaveOccurred())
			})

			It("Should return a nil config", func() {
				Expect(cfg).To(BeNil())
			})
		})

		Context("With a config file containing invalid yaml", func() {
			BeforeEach(func() {
				cfgFile = "config.invalid-yaml.yml"
			})

			It("Should return an error", func() {
				Expect(cfgErr).To(HaveOccurred())
			})

			It("Should return a nil config", func() {
				Expect(cfg).To(BeNil())
			})
		})

		Context("With a valid config file", func() {
			It("Should not return an error", func() {
				Expect(cfgErr).ToNot(HaveOccurred())
			})

			It("Should return a non-nil config", func() {
				Expect(cfg).ToNot(BeNil())
			})

			Describe("GetServices()", func() {
				It("Should return all of the services", func() {
					Expect(cfg.GetServices()).To(HaveLen(4))
				})
			})

			Describe("GetDefaultService()", func() {
				It("Should return a non-nil service", func() {
					Expect(cfg.GetDefaultService()).ToNot(BeNil())
				})

				It("Should return the default service", func() {
					Expect(cfg.GetDefaultService().Domain()).To(Equal("github.com"))
				})
			})

			Describe("GetService(name)", func() {
				It("Should return the service if it exists", func() {
					Expect(cfg.GetService("dev.azure.com")).ToNot(BeNil())
				})

				It("Should return the default service if no domain name is provided", func() {
					Expect(cfg.GetService("")).To(Equal(cfg.GetDefaultService()))
				})

				It("Should return nil if the service does not exist", func() {
					Expect(cfg.GetService("hubgitlabbucket.orgcom")).To(BeNil())
				})
			})

			Describe("GetApps()", func() {
				It("Should return all of the apps", func() {
					Expect(cfg.GetApps()).To(HaveLen(3))
				})
			})

			Describe("GetDefaultApp()", func() {
				It("Should return a non-nil app", func() {
					Expect(cfg.GetDefaultApp()).ToNot(BeNil())
				})

				It("Should return the default app", func() {
					Expect(cfg.GetDefaultApp().Name()).To(Equal("shell"))
				})
			})

			Describe("GetApp(name)", func() {
				It("Should return the app if it exists", func() {
					Expect(cfg.GetApp("code")).ToNot(BeNil())
				})

				It("Should return the default app if no domain name is provided", func() {
					Expect(cfg.GetApp("")).To(Equal(cfg.GetDefaultApp()))
				})

				It("Should return nil if the app does not exist", func() {
					Expect(cfg.GetApp("missingappname")).To(BeNil())
				})
			})

			Describe("GetAliases()", func() {
				It("Should get the full list of aliases", func() {
					Expect(cfg.GetAliases()).To(HaveLen(1))
				})
			})

			Describe("GetAlias(name)", func() {
				It("Should return the expansion of the alias if it exists", func() {
					Expect(cfg.GetAlias("gt")).To(Equal("github.com/SierraSoftworks/git-tool"))
				})

				It("Should return an empty string if the alias doesn't exist", func() {
					Expect(cfg.GetAlias("unknown")).To(BeEmpty())
				})
			})
		})
	})
})
