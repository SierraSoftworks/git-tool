package models

// A Service represents a git hosting service responsible for storing
// and serving repositories.
type Service interface {
	// The Domain is the canonical domain used to host repositories for
	// this service. For example: github.com or dev.azure.com
	Domain() string

	// The DirectoryGlob is used to determine how repository directories are
	// located on the file system within this service's repository tree.
	DirectoryGlob() string

	// Website fetches the HTTP(S) URL which may be used to view a web based
	// representation of a repository.
	Website(r Repo) string

	// HttpURL fetches the git+http URL which may be used to fetch or push
	// the repository's code.
	HttpURL(r Repo) string

	// GitURL fetches the git+ssh URL which may be used to fetch or push
	// the repository's code.
	GitURL(r Repo) string
}
