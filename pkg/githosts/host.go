package githosts

import "github.com/SierraSoftworks/git-tool/pkg/models"

// A Host is responsible for interacting with a specific git service
// provider to create and manage repositories on that service.
type Host interface {
	// Handles tells Git Tool whether this host type is able to handle
	// requests for a specific repository service.
	Handles(s models.Service) bool

	// HasRepo tells Git Tool whether a specific repository exists
	// on the remote git hosting service.
	HasRepo(r models.Repo) (bool, error)

	// CreateRepo attempts to create a repository on the remote
	// git hosting service.
	CreateRepo(r models.Repo) error
}
