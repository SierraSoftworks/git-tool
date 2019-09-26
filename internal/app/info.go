package app

import (
	"fmt"

	"github.com/SierraSoftworks/git-tool/internal/pkg/autocomplete"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/templates"
	"github.com/SierraSoftworks/git-tool/internal/pkg/tracing"
	"github.com/pkg/errors"
	"github.com/urfave/cli"
)

var repoInfoCommand = cli.Command{
	Name: "info",
	Aliases: []string{
		"i",
	},
	Usage:     "Gets the information pertaining to a specific repository.",
	ArgsUsage: "[repo]",
	Flags:     []cli.Flag{},
	Action: func(c *cli.Context) error {
		tracing.Enter("/app/command/info")
		defer tracing.Exit()

		repo, err := di.GetMapper().GetBestRepo(c.Args().First())
		if err != nil {
			return err
		}

		if repo == nil && c.NArg() == 0 {
			return errors.New("no repository specified")
		} else if repo == nil {
			return errors.New("could not find repository")
		}

		fmt.Fprintln(di.GetOutput(), templates.RepoFullInfo(repo))

		return nil
	},
	BashComplete: func(c *cli.Context) {
		tracing.Enter("/app/complete/info")
		defer tracing.Exit()

		if c.NArg() > 0 {
			return
		}

		cmp := autocomplete.NewCompleter(c.GlobalString("bash-completion-filter"))

		cmp.RepoAliases()
		cmp.DefaultServiceRepos()
		cmp.AllServiceRepos()
	},
}
