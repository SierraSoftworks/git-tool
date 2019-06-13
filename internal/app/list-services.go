package app

import (
	"fmt"

	"github.com/urfave/cli"
)

var listServicesCommand = cli.Command{
	Name:  "services",
	Usage: "Lists the services which are known to host git repos.",
	Action: func(c *cli.Context) error {
		for _, svc := range cfg.GetServices() {
			fmt.Printf("%s\n", svc.Domain())
		}

		return nil
	},
}
