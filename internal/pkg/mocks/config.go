package mocks

import (
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/pkg/models"
)

type Config struct {
	directory    string
	scratchpads  string
	services     []models.Service
	applications []models.App
	aliases      map[string]string
	features     di.Features
}

func NewConfig(base di.Config) *Config {
	return &Config{
		directory:    base.DevelopmentDirectory(),
		scratchpads:  base.ScratchDirectory(),
		services:     base.GetServices(),
		applications: base.GetApps(),
		aliases:      base.GetAliases(),
		features:     base.GetFeatures(),
	}
}

func (c *Config) DevelopmentDirectory() string {
	return c.directory
}

func (c *Config) ScratchDirectory() string {
	return c.scratchpads
}

func (c *Config) GetServices() []models.Service {
	return c.services
}

func (c *Config) GetService(domain string) models.Service {
	for _, s := range c.services {
		if s.Domain() == domain {
			return s
		}
	}

	return nil
}

func (c *Config) GetDefaultService() models.Service {
	if len(c.services) > 0 {
		return c.services[0]
	}

	return nil
}

func (c *Config) GetApps() []models.App {
	return c.applications
}

func (c *Config) GetApp(name string) models.App {
	for _, a := range c.applications {
		if a.Name() == name {
			return a
		}
	}
	return nil
}

func (c *Config) GetDefaultApp() models.App {
	if len(c.applications) > 0 {
		return c.applications[0]
	}

	return nil
}

func (c *Config) GetAliases() map[string]string {
	return c.aliases
}

func (c *Config) GetAlias(name string) string {
	return c.aliases[name]
}

func (c *Config) GetFeatures() di.Features {
	return c.features
}

func (c *Config) AddApp(a models.App) *Config {
	c.applications = append(c.applications, a)
	return c
}

func (c *Config) AddService(s models.Service) *Config {
	c.services = append(c.services, s)
	return c
}

func (c *Config) AddAlias(name, value string) *Config {
	c.aliases[name] = value
	return c
}

func (c *Config) SetFeatures(features di.Features) *Config {
	c.features = features
	return c
}
