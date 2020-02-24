package di

import (
	"github.com/SierraSoftworks/git-tool/internal/pkg/registry"
	"github.com/SierraSoftworks/git-tool/pkg/models"
)

var config Config

// SetConfig allows you to update the current config
func SetConfig(c Config) {
	config = c
}

// GetConfig gets the current configuration
func GetConfig() Config {
	return config
}

// Features are used to control the fine grained behaviour of Git Tool
type Features interface {
	UseNativeClone() bool
	CreateRemoteRepo() bool
	UseHttpTransport() bool
}

// The Config is used to configure the behavior of Git Tool
type Config interface {
	DevelopmentDirectory() string
	ScratchDirectory() string

	GetServices() []models.Service
	GetService(domain string) models.Service
	GetDefaultService() models.Service

	GetApps() []models.App
	GetApp(name string) models.App
	GetDefaultApp() models.App

	GetAliases() map[string]string
	GetAlias(name string) string

	GetFeatures() Features
	Update(entry registry.EntryConfig)
}
