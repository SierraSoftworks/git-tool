package config_test

import (
	"path/filepath"
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/test"
	testmodels "github.com/SierraSoftworks/git-tool/test/models"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestService(t *testing.T) {
	cfg, err := config.Load(test.GetTestDataPath("config.valid.yml"))
	require.NoError(t, err, "it should not return an error when loading the config")
	require.NotNil(t, cfg, "it should not return a nil config")

	t.Run("github.com", func(t *testing.T) {
		svc := cfg.GetService("github.com")

		repo := &testmodels.TestRepo{
			ModelFullName: "sierrasoftworks/test1",
			ModelService:  svc,
			ModelPath:     filepath.Join(test.GetTestPath(), svc.Domain(), "sierrasoftworks", "test1"),
			ModelValid:    false,
			ModelExists:   false,
		}

		if assert.NotNil(t, svc, "it should have github.com as a service") {
			assert.Equal(t, "github.com", svc.Domain(), "it should have the correct domain")
			assert.Equal(t, "*/*", svc.DirectoryGlob(), "it should have the correct directory glob for github.com")
			assert.Equal(t, "https://github.com/sierrasoftworks/test1", svc.Website(repo), "it should generate the correct web URL")
			assert.Equal(t, "git@github.com:sierrasoftworks/test1.git", svc.GitURL(repo), "it should generate the correct git+ssh URL")
			assert.Equal(t, "https://github.com/sierrasoftworks/test1.git", svc.HttpURL(repo), "it should generate the correct git+http URL")
		}
	})

	t.Run("gitlab.com", func(t *testing.T) {
		svc := cfg.GetService("gitlab.com")

		repo := &testmodels.TestRepo{
			ModelFullName: "sierrasoftworks/test1",
			ModelService:  svc,
			ModelPath:     filepath.Join(test.GetTestPath(), svc.Domain(), "sierrasoftworks", "test1"),
			ModelValid:    false,
			ModelExists:   false,
		}

		if assert.NotNil(t, svc, "it should have gitlab.com as a service") {
			assert.Equal(t, "gitlab.com", svc.Domain(), "it should have the correct domain")
			assert.Equal(t, "*/*", svc.DirectoryGlob(), "it should have the correct directory glob for gitlab.com")
			assert.Equal(t, "https://gitlab.com/sierrasoftworks/test1", svc.Website(repo), "it should generate the correct web URL")
			assert.Equal(t, "git@gitlab.com:sierrasoftworks/test1.git", svc.GitURL(repo), "it should generate the correct git+ssh URL")
			assert.Equal(t, "https://gitlab.com/sierrasoftworks/test1.git", svc.HttpURL(repo), "it should generate the correct git+http URL")
		}
	})

	t.Run("bitbucket.org", func(t *testing.T) {
		svc := cfg.GetService("bitbucket.org")

		repo := &testmodels.TestRepo{
			ModelFullName: "sierrasoftworks/test1",
			ModelService:  svc,
			ModelPath:     filepath.Join(test.GetTestPath(), svc.Domain(), "sierrasoftworks", "test1"),
			ModelValid:    false,
			ModelExists:   false,
		}

		if assert.NotNil(t, svc, "it should have bitbucket.org as a service") {
			assert.Equal(t, "bitbucket.org", svc.Domain(), "it should have the correct domain")
			assert.Equal(t, "*/*", svc.DirectoryGlob(), "it should have the correct directory glob for bitbucket.org")
			assert.Equal(t, "https://bitbucket.org/sierrasoftworks/test1", svc.Website(repo), "it should generate the correct web URL")
			assert.Equal(t, "git@bitbucket.org:sierrasoftworks/test1.git", svc.GitURL(repo), "it should generate the correct git+ssh URL")
			assert.Equal(t, "https://bitbucket.org/sierrasoftworks/test1.git", svc.HttpURL(repo), "it should generate the correct git+http URL")
		}
	})

	t.Run("dev.azure.com", func(t *testing.T) {
		svc := cfg.GetService("dev.azure.com")

		repo := &testmodels.TestRepo{
			ModelFullName: "sierrasoftworks/tests/test1",
			ModelService:  svc,
			ModelPath:     filepath.Join(test.GetTestPath(), svc.Domain(), "sierrasoftworks", "tests", "test1"),
			ModelValid:    false,
			ModelExists:   false,
		}

		if assert.NotNil(t, svc, "it should have dev.azure.com as a service") {
			assert.Equal(t, "dev.azure.com", svc.Domain(), "it should have the correct domain")
			assert.Equal(t, "*/*/*", svc.DirectoryGlob(), "it should have the correct directory glob for dev.azure.com")
			assert.Equal(t, "https://dev.azure.com/sierrasoftworks/tests/_git/test1", svc.Website(repo), "it should generate the correct web URL")
			assert.Equal(t, "git@ssh.dev.azure.com:v3/sierrasoftworks/tests/test1.git", svc.GitURL(repo), "it should generate the correct git+ssh URL")
			assert.Equal(t, "https://dev.azure.com/sierrasoftworks/tests/_git/test1", svc.HttpURL(repo), "it should generate the correct git+http URL")
		}
	})
}
