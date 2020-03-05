package app

import (
	"fmt"

	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/tracing"
	"github.com/urfave/cli/v2"
)

var listServicesCommand = &cli.Command{
	Name:  "services",
	Usage: "Lists the services which are known to host git repos.",
	Action: func(c *cli.Context) error {
		tracing.Enter("/app/command/services")
		defer tracing.Exit()

		for _, svc := range di.GetConfig().GetServices() {
			fmt.Fprintf(di.GetOutput(), "%s\n", svc.Domain())
		}

		return nil
	},
}
