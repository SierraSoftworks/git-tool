package di

import (
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

// The Config is used to configure the behavior of Git Tool
type Config interface {
	DevelopmentDirectory() string

	GetServices() []models.Service
	GetService(domain string) models.Service
	GetDefaultService() models.Service

	GetApps() []models.App
	GetApp(name string) models.App
	GetDefaultApp() models.App

	GetAliases() map[string]string
	GetAlias(name string) string
}