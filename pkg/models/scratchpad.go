package models

type Scratchpad interface {
	Target
	Exists() bool
}
