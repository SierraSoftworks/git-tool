package config

import (
	"io/ioutil"
	"os"

	"github.com/SierraSoftworks/git-tool/pkg/models"

	"github.com/go-yaml/yaml"
	"github.com/pkg/errors"
)

// The Config is used to configure the behavior of Git Tool
type Config interface {
	DevelopmentDirectory() string

	GetServices() []models.Service
	GetService(domain string) models.Service
	GetDefaultService() models.Service

	GetApps() []models.App
	GetApp(name string) models.App
	GetDefaultApp() models.App
}

type config struct {
	Directory string `json:"directory" yaml:"directory"`

	Services     []*service `json:"services" yaml:"services"`
	Applications []*app     `json:"apps" yaml:"apps"`
}

// Default gets a simple default configuration for Git Tool
// for environments where you have not defined a configuration
// file.
func Default() Config {
	return &config{
		Directory: os.Getenv("DEV_DIRECTORY"),
		Services: []*service{
			&service{
				DomainField:        "github.com",
				DirectoryGlobField: "*/*",
				WebsiteTemplate:    "https://{{ .Service.Domain }}/{{ .Repo.FullName }}",
				HttpUrlTemplate:    "https://{{ .Service.Domain }}/{{ .Repo.FullName }}.git",
				GitUrlTemplate:     "git@{{ .Service.Domain }}:{{ .Repo.FullName }}.git",
			},
		},
		Applications: []*app{
			&app{
				NameField:   "shell",
				CommandLine: "bash",
			},
		},
	}
}

// Load will attempt to load a configuration object from the provided file.
func Load(file string) (Config, error) {
	bytes, err := ioutil.ReadFile(file)
	if err != nil {
		return nil, errors.Wrap(err, "config: unable to read config file")
	}

	config := &config{}
	if err := yaml.Unmarshal(bytes, config); err != nil {
		return nil, errors.Wrap(err, "config: unable to parse config file")
	}

	return config, nil
}

func (c *config) DevelopmentDirectory() string {
	return c.Directory
}

func (c *config) GetApps() []models.App {
	apps := make([]models.App, len(c.Applications))
	for i, app := range c.Applications {
		apps[i] = app
	}

	return apps
}

func (c *config) GetServices() []models.Service {
	svcs := make([]models.Service, len(c.Services))
	for i, svc := range c.Services {
		svcs[i] = svc
	}

	return svcs
}

// GetDefaultService fetches the default configured service
func (c *config) GetDefaultService() models.Service {
	if len(c.Services) > 0 {
		return c.Services[0]
	}

	return nil
}

// GetService will retrieve a known service entry for the given service,
// if one exists in the config file, based on its domain name.
func (c *config) GetService(domain string) models.Service {
	if domain == "" {
		return c.GetDefaultService()
	}

	for _, s := range c.Services {
		if s.DomainField == domain {
			return s
		}
	}

	return nil
}

// GetDefaultApp fetches the default configured application
func (c *config) GetDefaultApp() models.App {
	if len(c.Applications) > 0 {
		return c.Applications[0]
	}

	return nil
}

// GetApp fetches the app whose name matches the provided name.
func (c *config) GetApp(name string) models.App {
	if name == "" {
		return c.GetDefaultApp()
	}

	for _, a := range c.Applications {
		if a.NameField == name {
			return a
		}
	}

	return nil
}
