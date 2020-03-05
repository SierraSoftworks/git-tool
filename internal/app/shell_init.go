package app

import (
	"fmt"

	"github.com/SierraSoftworks/git-tool/internal/pkg/autocomplete"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/tracing"
	"github.com/urfave/cli/v2"
)

var shellInitCommand = &cli.Command{
	Name:        "shell-init",
	Usage:       "Emits the script needed to configure your shell for use with Git-Tool.",
	Subcommands: cli.Commands{},
	Action: func(c *cli.Context) error {
		if c.NArg() > 0 {
			return nil
		}

		for _, shell := range autocomplete.GetInitScriptShells() {
			fmt.Fprintf(di.GetOutput(), " - %s\n", shell)
		}

		return nil
	},
	BashComplete: func(c *cli.Context) {
		tracing.Enter("/app/complete/shell-init")
		defer tracing.Exit()

		cmp := autocomplete.NewCompleter(c.String("bash-completion-filter"))
		cmp.Fixed(autocomplete.GetInitScriptShells()...)
	},
}

func init() {
	for _, shell := range autocomplete.GetInitScriptShells() {
		shell := shell
		shellInitCommand.Subcommands = append(shellInitCommand.Subcommands, &cli.Command{
			Name:        shell,
			Description: fmt.Sprintf("Prints the initialization script for %s", shell),
			Flags: []cli.Flag{
				&cli.BoolFlag{
					Name:   "full",
					Hidden: true,
				},
			},
			Action: func(c *cli.Context) error {
				if !c.Bool("full") {
					fmt.Fprint(di.GetOutput(), autocomplete.GetInitScript(shell))
				} else {
					fmt.Fprint(di.GetOutput(), autocomplete.GetFullInitScript(shell))
				}

				return nil
			},
		})
	}
}
