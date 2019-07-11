package config_test

import (
	"path/filepath"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/SierraSoftworks/git-tool/test"
	testmodels "github.com/SierraSoftworks/git-tool/test/models"

	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
)

var _ = Describe("Service", func() {
	getConfig := func() config.Config {
		cfg, err := config.Load(test.GetTestDataPath("config.valid.yml"))
		Expect(err).To(BeNil())
		Expect(cfg).ToNot(BeNil())

		return cfg
	}

	getRepo := func(svc models.Service) models.Repo {
		r := &testmodels.TestRepo{
			ModelFullName: "sierrasoftworks/test1",
			ModelService:  svc,
			ModelPath:     filepath.Join(test.GetTestPath(), svc.Domain(), "sierrasoftworks", "test1"),
			ModelValid:    false,
			ModelExists:   false,
		}

		return r
	}

	Describe("Domain()", func() {
		It("Should return the correct domain name when requested", func() {
			Expect(getConfig().GetService("github.com").Domain()).To(Equal("github.com"))
		})
	})

	Describe("DirectoryGlob()", func() {
		It("Should return the correct domain name when requested", func() {
			Expect(getConfig().GetService("github.com").DirectoryGlob()).To(Equal("*/*"))
			Expect(getConfig().GetService("dev.azure.com").DirectoryGlob()).To(Equal("*/*/*"))
		})
	})

	Context("github.com", func() {
		It("Should render the website address correctly", func() {
			svc := getConfig().GetService("github.com")
			repo := getRepo(svc)

			Expect(svc.Website(repo)).To(Equal("https://github.com/sierrasoftworks/test1"))
		})

		It("Should render the Git HTTP URL correctly", func() {
			svc := getConfig().GetService("github.com")
			repo := getRepo(svc)

			Expect(svc.HttpURL(repo)).To(Equal("https://github.com/sierrasoftworks/test1.git"))
		})

		It("Should render the Git SSH URL correctly", func() {
			svc := getConfig().GetService("github.com")
			repo := getRepo(svc)

			Expect(svc.GitURL(repo)).To(Equal("git@github.com:sierrasoftworks/test1.git"))
		})
	})

	Context("gitlab.com", func() {
		It("Should render the website address correctly", func() {
			svc := getConfig().GetService("gitlab.com")
			repo := getRepo(svc)

			Expect(svc.Website(repo)).To(Equal("https://gitlab.com/sierrasoftworks/test1"))
		})

		It("Should render the Git HTTP URL correctly", func() {
			svc := getConfig().GetService("gitlab.com")
			repo := getRepo(svc)

			Expect(svc.HttpURL(repo)).To(Equal("https://gitlab.com/sierrasoftworks/test1.git"))
		})

		It("Should render the Git SSH URL correctly", func() {
			svc := getConfig().GetService("gitlab.com")
			repo := getRepo(svc)

			Expect(svc.GitURL(repo)).To(Equal("git@gitlab.com:sierrasoftworks/test1.git"))
		})
	})

	Context("bitbucket.org", func() {
		It("Should render the website address correctly", func() {
			svc := getConfig().GetService("bitbucket.org")
			repo := getRepo(svc)

			Expect(svc.Website(repo)).To(Equal("https://bitbucket.org/sierrasoftworks/test1"))
		})

		It("Should render the Git HTTP URL correctly", func() {
			svc := getConfig().GetService("bitbucket.org")
			repo := getRepo(svc)

			Expect(svc.HttpURL(repo)).To(Equal("https://bitbucket.org/sierrasoftworks/test1.git"))
		})

		It("Should render the Git SSH URL correctly", func() {
			svc := getConfig().GetService("bitbucket.org")
			repo := getRepo(svc)

			Expect(svc.GitURL(repo)).To(Equal("git@bitbucket.org:sierrasoftworks/test1.git"))
		})
	})

	Context("dev.azure.com", func() {
		It("Should render the website address correctly", func() {
			svc := getConfig().GetService("dev.azure.com")
			repo := getRepo(svc)

			Expect(svc.Website(repo)).To(Equal("https://dev.azure.com/sierrasoftworks/_git/test1"))
		})

		It("Should render the Git HTTP URL correctly", func() {
			svc := getConfig().GetService("dev.azure.com")
			repo := getRepo(svc)

			Expect(svc.HttpURL(repo)).To(Equal("https://dev.azure.com/sierrasoftworks/_git/test1"))
		})

		It("Should render the Git SSH URL correctly", func() {
			svc := getConfig().GetService("dev.azure.com")
			repo := getRepo(svc)

			Expect(svc.GitURL(repo)).To(Equal("git@ssh.dev.azure.com:v3/sierrasoftworks/test1.git"))
		})
	})
})
