package app

import (
	"fmt"

	"github.com/urfave/cli"
)

var listAppsCommand = cli.Command{
	Name:  "apps",
	Usage: "Lists the applications which can be launched with the open command.",
	Action: func(c *cli.Context) error {
		for _, app := range cfg.GetApps() {
			fmt.Printf("%s\n", app.Name())
		}

		return nil
	},
}
