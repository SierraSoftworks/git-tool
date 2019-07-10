package app

import (
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/gitignore"
	"github.com/urfave/cli"
)

var getGitignoreCommand = cli.Command{
	Name: "gitignore",
	Aliases: []string{
		"ignore",
	},
	Usage:     "Fetches the latest version of a remote repository.",
	ArgsUsage: "[languages...]",
	Action: func(c *cli.Context) error {
		if c.NArg() == 0 {
			langs, err := gitignore.List()
			if err != nil {
				return err
			}

			for _, lang := range langs {
				di.GetOutput().Printf(" - %s\n", lang)
			}
		} else {
			ignore, err := gitignore.Ignore(append([]string{c.Args().First()}, c.Args().Tail()...)...)
			if err != nil {
				return err
			}

			di.GetOutput().Println(ignore)
		}

		return nil
	},
	BashComplete: func(c *cli.Context) {
		langs, err := gitignore.List()
		if err != nil {
			return
		}

		for _, lang := range langs {
			di.GetOutput().Println(lang)
		}
	},
}
