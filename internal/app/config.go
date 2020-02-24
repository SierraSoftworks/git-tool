package app

import (
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/tracing"
	"github.com/go-yaml/yaml"
	"github.com/pkg/errors"
	"github.com/urfave/cli"
)

var configCommand = cli.Command{
	Name:  "config",
	Usage: "Manages the Git Tool configuration file.",
	Action: func(c *cli.Context) error {
		tracing.Enter("/app/command/config")
		defer tracing.Exit()

		cfg := di.GetConfig()

		out, err := yaml.Marshal(cfg)
		if err != nil {
			return errors.Wrap(err, "config: unable to serialize config")
		}

		di.GetOutput().WriteString(string(out))

		return nil
	},
	Subcommands: cli.Commands{
		configListCommand,
		configAddCommand,
	},
}
