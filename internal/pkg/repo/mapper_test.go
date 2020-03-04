package repo_test

import (
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/mocks"
	"github.com/SierraSoftworks/git-tool/internal/pkg/repo"
	"github.com/SierraSoftworks/git-tool/test"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestMapper(t *testing.T) {
	cfg := mocks.NewConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))
	require.NotNil(t, cfg, "we should have a config")
	out := &mocks.Output{}
	launch := &mocks.Launcher{}
	init := &mocks.Initializer{}

	di.SetConfig(cfg)
	di.SetOutput(out)
	di.SetLauncher(launch)
	di.SetInitializer(init)
	di.SetMapper(&repo.Mapper{})

	reset := func() {
		cfg.Reset(config.DefaultForDirectory(test.GetTestPath("devdir")))
		out.Reset()
		launch.Reset()
		init.Reset()
	}

	t.Run("GetBestRepo()", func(t *testing.T) {
		t.Run("Aliases", func(t *testing.T) {
			reset()
			cfg.AddAlias("alias1", "github.com/sierrasoftworks/test1")
			cfg.AddAlias("alias2", "nonexistent.com/namespace/repo")

			t.Run("Known Service", func(t *testing.T) {
				repo, err := di.GetMapper().GetBestRepo("alias1")
				require.NoError(t, err, "it should not return an error")
				require.NotNil(t, repo, "it should not return a nil repo")

				assert.Equal(t, "github.com", repo.Service().Domain(), "it should be the correct service")
				assert.Equal(t, "sierrasoftworks/test1", repo.FullName(), "it should have the correct name")
			})

			t.Run("Unknown Service", func(t *testing.T) {
				repo, err := di.GetMapper().GetBestRepo("alias2")
				require.NoError(t, err, "it should not return an error")
				assert.Nil(t, repo, "it should return a nil repo")
			})

			t.Run("Unknown Alias", func(t *testing.T) {
				repo, err := di.GetMapper().GetBestRepo("alias3")
				require.NoError(t, err, "it should not return an error")
				assert.Nil(t, repo, "it should return a nil repo")
			})
		})

		t.Run("Fully Qualified Names", func(t *testing.T) {
			reset()

			t.Run("Known Service", func(t *testing.T) {
				repo, err := di.GetMapper().GetBestRepo("github.com/sierrasoftworks/test1")
				require.NoError(t, err, "it should not return an error")
				require.NotNil(t, repo, "it should not return a nil repo")

				assert.Equal(t, "github.com", repo.Service().Domain(), "it should be the correct service")
				assert.Equal(t, "sierrasoftworks/test1", repo.FullName(), "it should have the correct name")
			})

			t.Run("Unknown Service", func(t *testing.T) {
				repo, err := di.GetMapper().GetBestRepo("nonexistent.com/namespace/repo")
				require.NoError(t, err, "it should not return an error")
				assert.Nil(t, repo, "it should return a nil repo")
			})
		})

		t.Run("Default Service Names", func(t *testing.T) {
			reset()

			repo, err := di.GetMapper().GetBestRepo("sierrasoftworks/test1")
			require.NoError(t, err, "it should not return an error")
			require.NotNil(t, repo, "it should not return a nil repo")

			assert.Equal(t, "github.com", repo.Service().Domain(), "it should be the correct service")
			assert.Equal(t, "sierrasoftworks/test1", repo.FullName(), "it should have the correct name")
		})

		t.Run("Partial Names", func(t *testing.T) {
			t.Run("Single Match", func(t *testing.T) {
				repo, err := di.GetMapper().GetBestRepo("ghsstst1")
				require.NoError(t, err, "it should not return an error")
				require.NotNil(t, repo, "it should not return a nil repo")

				assert.Equal(t, "github.com", repo.Service().Domain(), "it should be the correct service")
				assert.Equal(t, "sierrasoftworks/test1", repo.FullName(), "it should have the correct name")
			})

			t.Run("Multiple Matches", func(t *testing.T) {
				repo, err := di.GetMapper().GetBestRepo("test")
				require.NoError(t, err, "it should not return an error")
				assert.Nil(t, repo, "it should return a nil repo")
			})

			t.Run("No Matches", func(t *testing.T) {
				repo, err := di.GetMapper().GetBestRepo("unrecognized")
				require.NoError(t, err, "it should not return an error")
				assert.Nil(t, repo, "it should return a nil repo")
			})
		})
	})
}
