package githosts

import (
	"github.com/SierraSoftworks/git-tool/pkg/models"
)

var hosts = []Host{}

// RegisterHost registers a new git hosting service with the service
// registry.
func RegisterHost(host Host) {
	hosts = append(hosts, host)
}

// GetHost fetches the Host instance responsible for handling a
// specific repository service type.
func GetHost(s models.Service) Host {
	for _, host := range hosts {
		if host.Handles(s) {
			return host
		}
	}

	return nil
}