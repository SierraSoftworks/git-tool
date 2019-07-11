package config_test

import (
	"os"
	"os/exec"
	"path/filepath"

	"github.com/SierraSoftworks/git-tool/pkg/models"
	testmodels "github.com/SierraSoftworks/git-tool/test/models"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/test"

	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
)

var _ = Describe("App", func() {
	getApp := func() models.App {
		cfg, err := config.Load(test.GetTestDataPath("config.valid.yml"))
		Expect(err).To(BeNil())
		Expect(cfg).ToNot(BeNil())

		app := cfg.GetApp("make")
		Expect(app).ToNot(BeNil())
		return app
	}

	getRepo := func() models.Repo {
		cfg, err := config.Load(test.GetTestDataPath("config.valid.yml"))
		Expect(err).To(BeNil())
		Expect(cfg).ToNot(BeNil())

		r := &testmodels.TestRepo{
			ModelFullName: "sierrasoftworks/test1",
			ModelService:  cfg.GetService("github.com"),
			ModelPath:     filepath.Join(test.GetTestPath(), "github.com", "sierrasoftworks", "test1"),
			ModelValid:    false,
			ModelExists:   false,
		}

		return r
	}

	Describe("Name()", func() {
		It("Should return the right name", func() {
			Expect(getApp().Name()).To(Equal("make"))
		})
	})

	Describe("GetCommand()", func() {
		getCmd := func() *exec.Cmd {
			cmd, err := getApp().GetCmd(getRepo())
			Expect(err).To(BeNil())
			Expect(cmd).ToNot(BeNil())

			return cmd
		}

		It("Should return a command", func() {
			cmd := getCmd()
			Expect(cmd).ToNot(BeNil())
		})

		Context("The command", func() {
			It("Should have the right directory", func() {
				Expect(getCmd().Dir).To(Equal(getRepo().Path()))
			})

			It("Should have the right path", func() {
				Expect(getCmd().Path).To(ContainSubstring("make"))
			})

			It("Should have the right arguments", func() {
				Expect(getCmd().Args).To(Equal([]string{"make", "build"}))
			})

			It("Should have the environment variables", func() {
				Expect(getCmd().Env).To(Equal(append(os.Environ(), "CI_SERVER=0", "REPO=sierrasoftworks/test1", "GITHOST=github.com")))
			})
		})
	})
})
