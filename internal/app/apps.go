package app

import (
	"fmt"

	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/tracing"
	"github.com/urfave/cli/v2"
)

var listAppsCommand = &cli.Command{
	Name:  "apps",
	Usage: "Lists the applications which can be launched with the open command.",
	Action: func(c *cli.Context) error {
		tracing.Enter("/app/command/apps")
		defer tracing.Exit()

		for _, app := range di.GetConfig().GetApps() {
			fmt.Fprintf(di.GetOutput(), "%s\n", app.Name())
		}

		return nil
	},
}
