package app

import (
	"fmt"
	"os"

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

			output := di.GetOutput()

			if o, ok := output.(interface {
				Stat() (os.FileInfo, error)
			}); ok {
				fi, err := o.Stat()
				if err == nil {
					if (fi.Mode() & os.ModeCharDevice) != 0 {
						// We're outputting to a terminal, we should redirect to the .gitignore file instead
						f, err := os.OpenFile(".gitignore", os.O_CREATE, os.ModePerm)
						if err == nil {
							defer f.Close()
							output = f
						}
					}
				}
			}

			fmt.Fprintln(output, ignore)
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
