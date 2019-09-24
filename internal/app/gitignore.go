package app

import (
	"fmt"

	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/gitignore"
	"github.com/urfave/cli"
)

var getGitignoreCommand = cli.Command{
	Name: "gitignore",
	Aliases: []string{
		"ignore",
	},
	Usage:     "Generates a .gitignore file for the provided languages.",
	ArgsUsage: "[languages...]",
	Action: func(c *cli.Context) error {
		if c.NArg() == 0 {
			langs, err := gitignore.List()
			if err != nil {
				return err
			}

			for _, lang := range langs {
				fmt.Fprintf(di.GetOutput(), " - %s\n", lang)
			}
		} else {
			ignore, err := gitignore.Ignore(append([]string{c.Args().First()}, c.Args().Tail()...)...)
			if err != nil {
				return err
			}

			fmt.Fprintln(di.GetOutput(), ignore)
		}

		return nil
	},
	BashComplete: func(c *cli.Context) {
		langs, err := gitignore.List()
		if err != nil {
			return
		}

		for _, lang := range langs {
			fmt.Fprintln(di.GetOutput(), lang)
		}
	},
}
