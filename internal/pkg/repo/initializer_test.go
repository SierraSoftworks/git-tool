package repo_test

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/mocks"
	"github.com/SierraSoftworks/git-tool/internal/pkg/repo"
	"github.com/SierraSoftworks/git-tool/test"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestInitializer(t *testing.T) {
	cfg := mocks.NewConfig(config.DefaultForDirectory(test.GetTestPath("devdir")))
	require.NotNil(t, cfg, "we should have a config")
	out := &mocks.Output{}

	di.SetConfig(cfg)
	di.SetOutput(out)
	di.SetLauncher(di.DefaultLauncher())
	di.SetInitializer(&repo.Initializer{})
	di.SetMapper(&repo.Mapper{})

	reset := func() {
		cfg.Reset(config.DefaultForDirectory(test.GetTestPath("devdir")))
		out.Reset()
	}

	t.Run("CreateScratchpad()", func(t *testing.T) {
		t.Run("Existing", func(t *testing.T) {
			reset()

			sp := repo.NewScratchpad("2019w15")
			assert.NoError(t, di.GetInitializer().CreateScratchpad(sp), "it should not return an error")
			assert.Empty(t, out.GetOperations(), "it should not log anything")
			assert.True(t, sp.Exists(), "it should leave the directory in place")
		})

		t.Run("New", func(t *testing.T) {
			reset()

			sp := repo.NewScratchpad("2019w01")
			defer os.RemoveAll(sp.Path())

			assert.NoError(t, di.GetInitializer().CreateScratchpad(sp), "it should not return an error")
			assert.Empty(t, out.GetOperations(), "it should not log anything")
			assert.True(t, sp.Exists(), "it should create the directory")
		})
	})

	t.Run("CreateRepository()", func(t *testing.T) {
		t.Run("Existing", func(t *testing.T) {
			reset()

			cfg.SetFeatures(&config.Features{
				NativeClone:  false,
				CreateRemote: false,
			})

			r := repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test2")
			defer os.RemoveAll(filepath.Join(r.Path(), ".git"))

			assert.True(t, r.Exists(), "the repo should exist to start with")
			assert.False(t, r.Valid(), "the repo should not be valid to start with")

			assert.NoError(t, di.GetInitializer().CreateRepository(r), "it should not return an error")
			assert.Empty(t, out.GetOperations(), "it should not have logged anything")
			assert.True(t, r.Exists(), "the repo should exist")
			assert.True(t, r.Valid(), "the repo should be initialized")
		})

		t.Run("New", func(t *testing.T) {
			reset()

			cfg.SetFeatures(&config.Features{
				NativeClone:  false,
				CreateRemote: false,
			})

			r := repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test3")
			defer os.RemoveAll(r.Path())

			assert.False(t, r.Exists(), "the repo should not exist to start with")

			assert.NoError(t, di.GetInitializer().CreateRepository(r), "it should not return an error")
			assert.Empty(t, out.GetOperations(), "it should not have logged anything")
			assert.True(t, r.Exists(), "the repo should exist")
			assert.True(t, r.Valid(), "the repo should be initialized")
		})
	})

	t.Run("CloneRepository()", func(t *testing.T) {
		runTest := func(t *testing.T, nativeClone bool) {
			t.Run("Existing", func(t *testing.T) {
				reset()

				cfg.SetFeatures(&config.Features{
					NativeClone:   nativeClone,
					CreateRemote:  false,
					HttpTransport: true,
				})

				r := repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/test1")
				defer os.RemoveAll(filepath.Join(r.Path(), ".git"))

				assert.True(t, r.Exists(), "the repo should exist to start with")
				assert.False(t, r.Valid(), "the repo should not be valid to start with")

				assert.NoError(t, di.GetInitializer().CloneRepository(r), "it should not return an error")
				assert.Empty(t, out.GetOperations(), "it should not have logged anything")
				assert.True(t, r.Exists(), "the repo should exist")
				assert.False(t, r.Valid(), "the repo should not have modified the repo")
			})

			t.Run("New", func(t *testing.T) {
				reset()

				cfg.SetFeatures(&config.Features{
					NativeClone:   nativeClone,
					CreateRemote:  false,
					HttpTransport: true,
				})

				r := repo.NewRepo(di.GetConfig().GetService("github.com"), "sierrasoftworks/licenses")
				os.RemoveAll(r.Path())
				defer os.RemoveAll(r.Path())

				assert.False(t, r.Exists(), "the repo should not exist to start with")

				assert.NoError(t, di.GetInitializer().CloneRepository(r), "it should not return an error")
				assert.NotEmpty(t, out.GetOperations(), "it should log the clone progress")
				assert.True(t, r.Exists(), "the repo should exist")
				assert.True(t, r.Valid(), "the repo should be valid")

			})
		}

		t.Run("Integrated Cloning", func(t *testing.T) {
			runTest(t, false)
		})

		t.Run("Native Cloning", func(t *testing.T) {
			runTest(t, true)
		})
	})
}
