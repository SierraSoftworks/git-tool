package app

import (
	"github.com/urfave/cli"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
)

var listServicesCommand = cli.Command{
	Name:  "services",
	Usage: "Lists the services which are known to host git repos.",
	Action: func(c *cli.Context) error {
		for _, svc := range di.GetConfig().GetServices() {
			di.GetOutput().Printf("%s\n", svc.Domain())
		}

		return nil
	},
}
