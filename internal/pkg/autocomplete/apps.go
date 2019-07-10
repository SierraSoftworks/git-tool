package autocomplete

import (
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
)

// Apps will generate autocomplete suggestions for applications in your config file.
func (c *Completer) Apps() {
	for _, app := range di.GetConfig().GetApps() {
		c.complete(app.Name())
	}
}
