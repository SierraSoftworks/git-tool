package app

import (
	"fmt"

	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/tracing"
	"github.com/pkg/errors"
	"github.com/urfave/cli/v2"
)

var configListCommand = &cli.Command{
	Name:  "list",
	Usage: "Lists the available configuration templates from the registry.",
	Aliases: []string{
		"ls",
	},
	Action: func(c *cli.Context) error {
		tracing.Enter("/app/command/config/list")
		defer tracing.Exit()

		entries, err := di.GetRegistry().GetEntries()
		if err != nil {
			return errors.Wrap(err, "registry: failed to retrieve config templates")
		}

		for _, e := range entries {
			di.GetOutput().WriteString(fmt.Sprintf("%s\n", e))
		}

		return nil
	},
}
