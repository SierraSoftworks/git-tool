package config_test

import (
	"os"
	"path/filepath"
	"testing"

	testmodels "github.com/SierraSoftworks/git-tool/test/models"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/test"
)

func TestApp(t *testing.T) {
	cfg, err := config.Load(test.GetTestDataPath("config.valid.yml"))
	require.NoError(t, err, "it should not return an error when loading the config")
	require.NotNil(t, cfg, "it should not return a nil config")

	app := cfg.GetApp("make")
	require.NotNil(t, app, "it should fetch the 'make' app successfully")

	repo := &testmodels.TestRepo{
		ModelFullName: "sierrasoftworks/test1",
		ModelService:  cfg.GetService("github.com"),
		ModelPath:     filepath.Join(test.GetTestPath(), "github.com", "sierrasoftworks", "test1"),
		ModelValid:    false,
		ModelExists:   false,
	}

	t.Run("Name()", func(t *testing.T) {
		assert.Equal(t, app.Name(), "make", "it should return the correct app name")
	})

	t.Run("GetCommand()", func(t *testing.T) {
		cmd, err := app.GetCmd(repo)
		require.NoError(t, err, "it should not return an error")
		require.NotNil(t, cmd, "it should return a valid command")

		assert.Equal(t, cmd.Dir, repo.Path(), "it should be in the correct directory")
		assert.Contains(t, cmd.Path, "make", "it should use the correct command path")
		assert.Equal(t, cmd.Args, []string{"make", "build"}, "it should use the correct command arguments")
		assert.Equal(t, cmd.Env, append(os.Environ(), "CI_SERVER=0", "REPO=sierrasoftworks/test1", "GITHOST=github.com"), "it should use the correct environment variables")
	})
}
