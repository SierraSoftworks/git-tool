package models

// A Target represents a location where which an application may be launched
type Target interface {
	// Name gets the name
	Name() string

	// Path gets the location on the local filesystem where this target may be found.
	Path() string

	// TemplateContext gets the context used to render templates for this target type.
	TemplateContext() *TemplateContext
}

// TemplateContext is used for the rendering of all templates for target objects
type TemplateContext struct {
	Target     Target
	Repo       Repo
	Scratchpad Scratchpad
	Service    Service
}
