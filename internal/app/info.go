package app

import (
	"github.com/SierraSoftworks/git-tool/internal/pkg/autocomplete"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/templates"
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
		repo, err := di.GetMapper().GetBestRepo(c.Args().First())
		if err != nil {
			return err
		}

		if repo == nil {
			return errors.New("could not find repository")
		}

		di.GetOutput().Println(templates.RepoFullInfo(repo))

		return nil
	},
	BashComplete: func(c *cli.Context) {
		if c.NArg() > 0 {
			return
		}

		cmp := autocomplete.NewCompleter(c.GlobalString("bash-completion-filter"))

		cmp.RepoAliases()
		cmp.DefaultServiceRepos()
		cmp.AllServiceRepos()
	},
}
