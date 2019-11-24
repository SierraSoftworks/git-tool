package registry

import (
	"runtime"
)

// Entry represents a registry entry which may be added to your configuration.
type Entry struct {
	Name        string        `json:"name" yaml:"name"`
	Description string        `json:"description" yaml:"description"`
	Configs     []EntryConfig `json:"configs" yaml:"configs"`
}

// EntryConfig is a platform-specific configuration which should be applied for this entry.
type EntryConfig struct {
	Platform string    `json:"platform" yaml:"platform"`
	App      *EntryApp `json:"app,omitempty" yaml:"app,omitempty"`
}

// IsCompatible determines whether this EntryConfig is compatible with
// the current platform.
func (e *EntryConfig) IsCompatible() bool {
	if e.Platform == "any" {
		return true
	}

	if e.Platform == runtime.GOOS {
		return true
	}

	return false
}

// EntryApp is used to configure an application for a user.
type EntryApp struct {
	Name        string   `json:"name" yaml:"name"`
	Command     string   `json:"command" yaml:"command"`
	Arguments   []string `json:"arguments" yaml:"arguments"`
	Environment []string `json:"environment" yaml:"environment"`
}

// EntryService is used to configure a new service for a user.
type EntryService struct {
	Domain string `json:"domain" yaml:"domain"`

	Website string `json:"website" yaml:"website"`
	HTTPURL string `json:"httpUrl" yaml:"httpUrl"`
	GitURL  string `json:"gitUrl" yaml:"gitUrl"`

	Pattern string `json:"pattern" yaml:"pattern"`
}
