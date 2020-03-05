package app

import (
	"fmt"

	"github.com/SierraSoftworks/git-tool/internal/pkg/autocomplete"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/tracing"
	"github.com/pkg/errors"
	"github.com/urfave/cli/v2"
)

var configAddCommand = &cli.Command{
	Name:  "add",
	Usage: "Adds a configuration template from the registry to your config file.",
	Action: func(c *cli.Context) error {
		tracing.Enter("/app/command/config/add")
		defer tracing.Exit()

		if c.String("config") == "" {
			return errors.New("usage: you must set the GITTOOL_CONFIG environment variable or pass the --config argument")
		}

		if c.NArg() < 1 {
			return errors.New("usage: you must provide the ID of the config template")
		}

		entry, err := di.GetRegistry().GetEntry(c.Args().First())
		if err != nil {
			return err
		}

		if entry != nil {
			di.GetOutput().WriteString(fmt.Sprintf("Applying %s\n", c.Args().First()))
			di.GetOutput().WriteString(fmt.Sprintf("Name:  %s\n", entry.Name))
			di.GetOutput().WriteString(fmt.Sprintf("About: %s\n", entry.Description))

			for _, e := range entry.Configs {
				if e.IsCompatible() {
					di.GetConfig().Update(e)
				}
			}

			return di.GetConfig().Save(c.String("config"))
		}

		return nil
	},
	BashComplete: func(c *cli.Context) {
		tracing.Enter("/app/complete/config/add")
		defer tracing.Exit()

		cmp := autocomplete.NewCompleter(c.String("bash-completion-filter"))

		if c.NArg() == 0 {
			cmp.Fixed("apps/", "services/")
		}
	},
}
