package repo_test

import (
	"os"
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/mocks"
	"github.com/SierraSoftworks/git-tool/internal/pkg/repo"
	"github.com/SierraSoftworks/git-tool/internal/pkg/tasks"
	"github.com/SierraSoftworks/git-tool/test"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestGetBranches(t *testing.T) {
	cfg := mocks.NewConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))
	require.NotNil(t, cfg, "we should have a config")

	di.SetConfig(cfg)
	di.SetMapper(&repo.Mapper{})

	svc := cfg.GetService("github.com")
	require.NotNil(t, svc, "the service should not be nil")

	t.Run("Missing Repo", func(t *testing.T) {
		branches, err := di.GetMapper().GetBranches(repo.NewRepo(svc, "sierrasoftworks/test3"))
		assert.Nil(t, branches, "it should not return any branches")
		assert.Error(t, err, "it should return an error")
	})

	t.Run("Invalid Repo", func(t *testing.T) {
		branches, err := di.GetMapper().GetBranches(repo.NewRepo(svc, "sierrasoftworks/test1"))
		assert.Nil(t, branches, "it should not return any branches")
		assert.Error(t, err, "it should return an error")
	})

	t.Run("Valid Repo", func(t *testing.T) {
		r := repo.NewRepo(svc, "sierrasoftworks/test_get_branches")
		defer os.RemoveAll(r.Path())

		require.NoError(t, tasks.Sequence(
			tasks.NewFolder(),
			tasks.GitInit(),
			tasks.GitRemote("origin"),
			tasks.NewFile("README.md", []byte("# Test Repo")),
			tasks.GitCommit("Initial Commit", "README.md"),
			tasks.GitNewRef("refs/remotes/origin/test-branch"),
			tasks.NewFile("README.md", []byte("# Test Repo\nWith changes")),
			tasks.GitCommit("Made changes to README", "README.md"),
			tasks.GitNewRef("refs/remotes/origin/master"),
			tasks.GitNewRef("refs/heads/test-branch2"),
			tasks.GitCheckout("master", false),
		).ApplyRepo(r), "we should be able to setup the test repo")

		branches, err := di.GetMapper().GetBranches(r)
		require.NoError(t, err, "it should not return an error")
		require.NotNil(t, branches, "it should return at least one branch")

		assert.Contains(t, branches, "test-branch2", "it should contain a branch which exists locally")
		assert.Contains(t, branches, "test-branch", "it should contain a branch which exists remotely")
		assert.Contains(t, branches, "master", "it should contain a branch which exists both locally and remotely")
		assert.Contains(t, branches, "origin/master", "it should contain remote branch names")
		assert.Contains(t, branches, "origin/test-branch", "it should contain remote branch names")
	})
}
