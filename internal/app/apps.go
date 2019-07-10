package app

import (
	"github.com/urfave/cli"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
)

var listAppsCommand = cli.Command{
	Name:  "apps",
	Usage: "Lists the applications which can be launched with the open command.",
	Action: func(c *cli.Context) error {
		for _, app := range di.GetConfig().GetApps() {
			di.GetOutput().Printf("%s\n", app.Name())
		}

		return nil
	},
}
