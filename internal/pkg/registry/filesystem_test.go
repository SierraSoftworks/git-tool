package registry_test

import (
	"path/filepath"
	"strings"
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/registry"
	"github.com/SierraSoftworks/git-tool/test"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestFilesystemSource(t *testing.T) {
	s := registry.FileSystem(filepath.Join(test.GetTestDataPath(), "registry"))
	require.NotNil(t, s, "the registry should not be nil")

	t.Run("GetEntries()", func(t *testing.T) {
		entries, err := s.GetEntries()
		require.NoError(t, err, "there should not be an error getting the entries")

		require.NotEmpty(t, entries, "there should be at least one entry")
		assert.Contains(t, entries, "apps/app", "it should contain the app entry")
		assert.Contains(t, entries, "services/service", "it should contain the service entry")
	})

	t.Run("GetEntry()", func(t *testing.T) {
		t.Run("GetEntry(apps/app)", func(t *testing.T) {
			entry, err := s.GetEntry("apps/app")
			require.NoError(t, err, "there should not be an error getting an entry")

			require.NotNil(t, entry, "the resulting entry should not be nil")
			assert.Equal(t, "Test App", entry.Name, "the resulting entry should have the right name")
			assert.Equal(t, "This is a test app.", entry.Description, "the resulting entry should have a description")
			assert.NotEmpty(t, entry.Configs, "the resulting entry should have configs")

			for _, cfg := range entry.Configs {
				assert.Equal(t, "any", cfg.Platform, "the entry should have the correct platform specified")
				if assert.NotNil(t, cfg.App, "the entry should have an app config") {
					assert.Equal(t, "echo", cfg.App.Command, "the entry app command should be correct")
					assert.Equal(t, []string{"Hello $USER"}, cfg.App.Arguments, "the entry app arguments should be correct")
					assert.Equal(t, []string{"USER=test"}, cfg.App.Environment, "the entry app environment should be correct")
				}
			}
		})

		
		t.Run("GetEntry(services/service)", func(t *testing.T) {
			entry, err := s.GetEntry("services/service")
			require.NoError(t, err, "there should not be an error getting an entry")

			require.NotNil(t, entry, "the resulting entry should not be nil")
			assert.Equal(t, "Test Service", entry.Name, "the resulting entry should have the right name")
			assert.Equal(t, "This is a test service.", entry.Description, "the resulting entry should have a description")
			assert.NotEmpty(t, entry.Configs, "the resulting entry should have configs")

			for _, cfg := range entry.Configs {
				assert.Equal(t, "any", cfg.Platform, "the entry should have the correct platform specified")
				if assert.NotNil(t, cfg.Service, "the entry should have a service config") {
					assert.Equal(t, "test.example.com", cfg.Service.Domain, "the entry service domain should be correct")
					assert.Equal(t, "*/*", cfg.Service.Pattern, "the entry service repo pattern should be correct")
					assert.Equal(t, "https://{{ .Service.Domain }}/{{ .Repo.FullName }}", cfg.Service.Website, "the entry service website should be correct")
					assert.Equal(t, "git@{{ .Service.Domain }}:{{ .Repo.FullName }}.git", cfg.Service.GitURL, "the entry service git URL template should be correct")
					assert.Equal(t, "https://{{ .Service.Domain }}/{{ .Repo.FullName }}.git", cfg.Service.HTTPURL, "the entry service git+HTTP URL template should be correct")
				}
			}
		})
	})
}

func TestRegistryCompliance(t *testing.T) {
	s := registry.FileSystem(filepath.Join(test.GetProjectRoot(), "registry"))
	require.NotNil(t, s, "the registry should not be nil")

	entries, err := s.GetEntries()
	require.NoError(t, err, "there should be no errors getting the entries")

	for _, entryName := range entries {
		isApp := strings.HasPrefix(entryName, "apps/")
		isService := strings.HasPrefix(entryName, "services/")

		assert.True(t, isApp || isService, "the entry should be in either the apps or services directory")

		entryName := entryName
		t.Run(entryName, func(t *testing.T) {
			entry, err := s.GetEntry(entryName)
			require.NoError(t, err, "we should be able to load the entry")

			assert.NotEmpty(t, entry.Name, "the entry name should not be empty")
			assert.NotEmpty(t, entry.Description, "the entry description should not be empty")

			if assert.NotNil(t, entry.Configs, "the entry configs should not be nil") {
				assert.NotEmpty(t, entry.Configs, "the entry should have at least one config")

				for _, cfg := range entry.Configs {
					if assert.NotNil(t, cfg, "the configs in the entry should not be nil") {
						assert.NotEmpty(t, cfg.Platform, "the config should have a platform specified")

						if isApp {
							assert.NotNil(t, cfg.App, "app entries should have an app config")
							assert.NotEmpty(t, cfg.App.Name, "app entries should have a name")
							assert.NotEmpty(t, cfg.App.Command, "app entries should have a command")
						}

						if isService {
							assert.NotNil(t, cfg.Service, "service entries should have a service config")
							assert.NotEmpty(t, cfg.Service.Domain, "service entries should have a non-empty domain name")
							if assert.NotEmpty(t, cfg.Service.Pattern, "service entries should have a non-empty pattern") {
								assert.Regexp(t, "\\*(/\\*)*", cfg.Service.Pattern, "service entry patterns should match the regex /\\*(\\/\\*)*/")
							}

							assert.NotEmpty(t, cfg.Service.Website, "service entries should have a non-empty repo website template")
							assert.NotEmpty(t, cfg.Service.GitURL, "service entries should have a non-empty git URL template")
							assert.NotEmpty(t, cfg.Service.HTTPURL, "service entries should have a non-empty GIT+HTTP URL template")
						}
					}
				}
			}
		})
	}
}
