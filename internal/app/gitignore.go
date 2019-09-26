package app

import (
	"fmt"
	"os"

	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/gitignore"
	"github.com/SierraSoftworks/git-tool/internal/pkg/tracing"
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
		tracing.Enter("/app/command/gitignore")
		defer tracing.Exit()

		if c.NArg() == 0 {
			tracing.Transition("/app/command/gitignore/list")
			langs, err := gitignore.List()
			if err != nil {
				return err
			}

			for _, lang := range langs {
				fmt.Fprintf(di.GetOutput(), " - %s\n", lang)
			}
		} else {
			tracing.Transition("/app/command/gitignore/ignore")
			languages := append([]string{c.Args().First()}, c.Args().Tail()...)

			output := di.GetOutput()

			if o, ok := output.(interface {
				Stat() (os.FileInfo, error)
			}); ok {
				fi, err := o.Stat()
				if err == nil {
					if (fi.Mode() & os.ModeCharDevice) != 0 {
						tracing.Transition("/app/command/gitignore/ignore/file")
						// We're outputting to a terminal, we should redirect to the .gitignore file instead
						return gitignore.AddOrUpdate(".gitignore", languages...)
					}
				}
			}

			tracing.Transition("/app/command/gitignore/ignore/stdout")
			ignore, err := gitignore.Ignore(languages...)
			if err != nil {
				return err
			}

			fmt.Fprintln(output, ignore)
		}

		return nil
	},
	BashComplete: func(c *cli.Context) {
		tracing.Enter("/app/complete/gitignore")
		defer tracing.Exit()

		langs, err := gitignore.List()
		if err != nil {
			return
		}

		for _, lang := range langs {
			fmt.Fprintln(di.GetOutput(), lang)
		}
	},
}
