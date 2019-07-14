package models

// A Repo represents a specific git repository
type Repo interface {
	Target

	// FullName gets the full name of the repository including its namespace
	FullName() string
	// Namespace gets the portion of the repository's full name prior to its final short name segment.
	Namespace() string
	
	// Service retrieves the details of the service hosting this repository
	Service() Service
	
	Website() string
	GitURL() string
	HttpURL() string

	// Exists checks whether a repository entry is present on the local filesystem
	// at the expected path.
	Exists() bool

	// Valid checks whether the current repo is initialized correctly.
	Valid() bool
}