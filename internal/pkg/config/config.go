package config

import (
	"io/ioutil"
	"os"
	"path/filepath"

	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/registry"
	"github.com/SierraSoftworks/git-tool/pkg/models"

	"github.com/go-yaml/yaml"
	"github.com/pkg/errors"
)

type config struct {
	Directory   string `json:"directory" yaml:"directory"`
	Scratchpads string `json:"scratchpads" yaml:"scratchpads"`

	Services     []*service        `json:"services" yaml:"services"`
	Applications []*app            `json:"apps" yaml:"apps"`
	Aliases      map[string]string `json:"aliases" yaml:"aliases"`

	Features *Features `json:"features" yaml:"features"`
}

// Default gets a simple default configuration for Git Tool
// for environments where you have not defined a configuration
// file.
func Default() di.Config {
	return DefaultForDirectory(os.Getenv("DEV_DIRECTORY"))
}

// DefaultForDirectory gets a simple default configuration for Git Tool
// pointed at a specific development directory.
func DefaultForDirectory(dir string) di.Config {
	return &config{
		Directory: dir,
		Services: []*service{
			&service{
				DomainField:        "github.com",
				DirectoryGlobField: "*/*",
				WebsiteTemplate:    "https://{{ .Service.Domain }}/{{ .Repo.FullName }}",
				HttpUrlTemplate:    "https://{{ .Service.Domain }}/{{ .Repo.FullName }}.git",
				GitUrlTemplate:     "git@{{ .Service.Domain }}:{{ .Repo.FullName }}.git",
			},
			&service{
				DomainField:        "dev.azure.com",
				DirectoryGlobField: "*/*/*",
				WebsiteTemplate:    "https://{{ .Service.Domain }}/{{ .Repo.Namespace }}/_git/{{ .Repo.Name }}",
				HttpUrlTemplate:    "https://{{ .Service.Domain }}/{{ .Repo.Namespace }}/_git/{{ .Repo.Name }}",
				GitUrlTemplate:     "git@ssh.{{ .Service.Domain }}:v3/{{ .Repo.FullName }}.git",
			},
		},
		Applications: []*app{
			&app{
				NameField:   "shell",
				CommandLine: "bash",
			},
		},
		Aliases:  map[string]string{},
		Features: defaultFeatures(),
	}
}

// Load will attempt to load a configuration object from the provided file.
func Load(file string) (di.Config, error) {
	bytes, err := ioutil.ReadFile(file)
	if err != nil {
		return nil, errors.Wrap(err, "config: unable to read config file")
	}

	config := Default()
	if err := yaml.Unmarshal(bytes, config); err != nil {
		return nil, errors.Wrap(err, "config: unable to parse config file")
	}

	return config, nil
}

func (c *config) DevelopmentDirectory() string {
	return c.Directory
}

func (c *config) ScratchDirectory() string {
	if c.Scratchpads == "" {
		return filepath.Join(c.Directory, "scratch")
	}

	return c.Scratchpads
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

func (c *config) GetAliases() map[string]string {
	return c.Aliases
}

func (c *config) GetAlias(name string) string {
	return c.Aliases[name]
}

func (c *config) GetFeatures() di.Features {
	return c.Features
}

func (c *config) Update(entry registry.EntryConfig) {
	if entry.App != nil {
		c.Applications = append(c.Applications, &app{
			NameField:   entry.App.Name,
			CommandLine: entry.App.Command,
			Arguments:   entry.App.Arguments,
			Environment: entry.App.Environment,
		})
	}

	if entry.Service != nil {
		c.Services = append(c.Services, &service{
			DomainField:        entry.Service.Domain,
			WebsiteTemplate:    entry.Service.Website,
			HttpUrlTemplate:    entry.Service.HTTPURL,
			GitUrlTemplate:     entry.Service.GitURL,
			DirectoryGlobField: entry.Service.Pattern,
		})
	}
}

func (c *config) Save(path string) error {
	out, err := yaml.Marshal(c)
	if err != nil {
		return errors.Wrap(err, "config: unable to serialize config")
	}

	return errors.Wrap(ioutil.WriteFile(path, out, os.ModePerm), "config: unable to save config")
}
