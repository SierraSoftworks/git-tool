package app

import (
	"fmt"

	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/urfave/cli"
)

var listAppsCommand = cli.Command{
	Name:  "apps",
	Usage: "Lists the applications which can be launched with the open command.",
	Action: func(c *cli.Context) error {
		for _, app := range di.GetConfig().GetApps() {
			fmt.Fprintf(di.GetOutput(), "%s\n", app.Name())
		}

		return nil
	},
}
