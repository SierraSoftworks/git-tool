package models

type Scratchpad interface {
	Target
	Name() string
	Exists() bool
}
