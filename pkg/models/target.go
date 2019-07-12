package models

// A Target represents a location where which an application may be launched
type Target interface {
	// Path gets the location on the local filesystem where this target may be found.
	Path() string

	// TemplateContext gets the context used to render templates for this target type.
	TemplateContext() interface{}
}
