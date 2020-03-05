package config_test

import (
	"os"
	"testing"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/registry"
	"github.com/SierraSoftworks/git-tool/test"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestConfig(t *testing.T) {
	os.Setenv("DEV_DIRECTORY", test.GetTestPath())

	t.Run("Default()", func(t *testing.T) {
		cfg := config.Default()

		require.NotNil(t, cfg, "it should return a default config")
		assert.Equal(t, cfg.DevelopmentDirectory(), test.GetTestPath(), "it should have the right development directory")

		assert.NotEmpty(t, cfg.GetServices(), "it should have some services")
		assert.NotNil(t, cfg.GetDefaultService(), "it should have a default service")

		assert.NotEmpty(t, cfg.GetApps(), "it should have some apps")
		assert.NotNil(t, cfg.GetApp("shell"), "it should have a shell app by default")
		if assert.NotNil(t, cfg.GetDefaultApp(), "it should have a default app") {
			assert.Equal(t, cfg.GetDefaultApp().Name(), "shell", "it should use shell as the default app")
		}
	})

	t.Run("DefaultForDirectory()", func(t *testing.T) {
		cfg := config.DefaultForDirectory(test.GetTestPath())

		require.NotNil(t, cfg, "it should return a default config")
		assert.Equal(t, cfg.DevelopmentDirectory(), test.GetTestPath(), "it should have the right development directory")

		assert.NotEmpty(t, cfg.GetServices(), "it should have some services")
		assert.NotNil(t, cfg.GetDefaultService(), "it should have a default service")

		assert.NotEmpty(t, cfg.GetApps(), "it should have some apps")
		assert.NotNil(t, cfg.GetApp("shell"), "it should have a shell app by default")
		if assert.NotNil(t, cfg.GetDefaultApp(), "it should have a default app") {
			assert.Equal(t, cfg.GetDefaultApp().Name(), "shell", "it should use shell as the default app")
		}
	})

	t.Run("Load()", func(t *testing.T) {
		cfg, err := config.Load(test.GetTestDataPath("config.valid.yml"))
		require.NoError(t, err, "it should not return an error")
		require.NotNil(t, cfg, "it should not return a nil config")

		assert.Len(t, cfg.GetServices(), 4, "it should have 4 services")

		if assert.NotNil(t, cfg.GetDefaultService(), "it should have a default service") {
			assert.Equal(t, cfg.GetDefaultService().Domain(), "github.com", "it should return the correct default service")
			assert.Equal(t, cfg.GetService(""), cfg.GetDefaultService(), "it should return the default service if no name is provided")
		}

		assert.NotNil(t, cfg.GetService("dev.azure.com"), "it should return a service by its domain")
		assert.Nil(t, cfg.GetService("hubgitlabbucket.orgcom"), "it should return nil if an unknown service is requested")

		assert.Len(t, cfg.GetApps(), 3, "it should have 3 apps")

		if assert.NotNil(t, cfg.GetDefaultApp(), "it should have a default app") {
			assert.Equal(t, cfg.GetDefaultApp().Name(), "shell", "it should have the correct default app")
			assert.Equal(t, cfg.GetApp(""), cfg.GetDefaultApp(), "it should return the default app if no name is provided")
		}

		assert.NotNil(t, cfg.GetApp("code"), "it should return an app by its name")
		assert.Nil(t, cfg.GetApp("missingappname"), "it should return nil if an unknown app is requested")

		assert.Len(t, cfg.GetAliases(), 1, "it should have an alias")
		assert.Equal(t, cfg.GetAlias("gt"), "github.com/SierraSoftworks/git-tool", "it should return an alias's expansion")
		assert.Empty(t, cfg.GetAlias("unknown"), "it should return an empty string if an unknown alias is provided")

		if assert.NotNil(t, cfg.GetFeatures(), "it should return a non-nil features object") {
			assert.False(t, cfg.GetFeatures().UseNativeClone(), "it should have native cloning disabled")
		}

		t.Run("Missing File", func(t *testing.T) {
			cfg, err := config.Load(test.GetTestDataPath("config.missing.yml"))
			assert.Error(t, err, "it should return an error")
			assert.Nil(t, cfg, "it should return a nil config")
		})

		t.Run("Invalid YAML", func(t *testing.T) {
			cfg, err := config.Load(test.GetTestDataPath("config.invalid-yaml.yml"))
			assert.Error(t, err, "it should return an error")
			assert.Nil(t, cfg, "it should return a nil config")
		})
	})

	t.Run("Update()", func(t *testing.T) {
		cfg := config.Default()
		require.NotNil(t, cfg, "it should return a default config")

		t.Run("Empty Update", func(t *testing.T) {
			cfg = config.Default()
			cfg.Update(registry.EntryConfig{
				Platform: "any",
			})

			assert.Equal(t, cfg.GetApps(), config.Default().GetApps(), "it should not have introduced any new apps")
			assert.Equal(t, cfg.GetServices(), config.Default().GetServices(), "it should not have introduced any new services")
		})

		t.Run("New App", func(t *testing.T) {
			cfg = config.Default()
			cfg.Update(registry.EntryConfig{
				Platform: "any",
				App: &registry.EntryApp{
					Name:    "test",
					Command: "/bin/sh",
					Arguments: []string{
						"-c",
						"echo $MESSAGE",
					},
					Environment: []string{
						"MESSAGE=Test",
					},
				},
			})

			assert.Equal(t, config.Default().GetServices(), cfg.GetServices(), "it should not have introduced any new services")
			assert.Len(t, cfg.GetApps(), len(config.Default().GetApps())+1, "it should have introduced a new app")
			assert.NotNil(t, cfg.GetApp("test"), "it should have added the test app")
		})

		t.Run("New Service", func(t *testing.T) {
			cfg = config.Default()
			cfg.Update(registry.EntryConfig{
				Platform: "any",
				Service: &registry.EntryService{
					Domain:  "test.example.com",
					Website: "https://test.example.com/{{ .Repo.Namespace }}/{{ .Repo.Name }}",
					HTTPURL: "https://test.example.com/{{ .Repo.Namespace }}/{{ .Repo.Name }}.git",
					GitURL:  "git@test.example.com:{{ .Repo.Namespace }}/{{ .Repo.Name }}",
					Pattern: "*/*",
				},
			})

			assert.Equal(t, config.Default().GetApps(), cfg.GetApps(), "it should not have introduced any new apps")
			assert.Len(t, cfg.GetServices(), len(config.Default().GetServices())+1, "it should have added a new service")
			assert.NotNil(t, cfg.GetService("test.example.com"), "it should have added the test service")
		})
	})
}
