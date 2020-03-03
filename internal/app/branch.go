package app

import (
	"github.com/SierraSoftworks/git-tool/internal/pkg/autocomplete"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/tasks"
	"github.com/SierraSoftworks/git-tool/internal/pkg/tracing"
	"github.com/pkg/errors"
	"github.com/urfave/cli"
)

var branchCommand = cli.Command{
	Name: "branch",
	Aliases: []string{
		"b",
	},
	Usage: "Checks out a branch with the given name from the current repository.",
	Action: func(c *cli.Context) error {
		tracing.Enter("/app/command/branch")
		defer tracing.Exit()

		repo, err := di.GetMapper().GetCurrentDirectoryRepo()
		if err != nil {
			return err
		}

		if repo == nil {
			return errors.New("usage: command must be run from within a repository")
		}

		if c.NArg() < 1 {
			return errors.New("usage: command requires a branch to be provided")
		}

		return tasks.GitCheckout(c.Args().First(), false).ApplyRepo(repo)
	},
	BashComplete: func(c *cli.Context) {
		repo, err := di.GetMapper().GetCurrentDirectoryRepo()
		if err != nil {
			return
		}

		if repo == nil {
			return
		}

		branches, err := di.GetMapper().GetBranches(repo)
		if err != nil {
			return
		}

		cmp := autocomplete.NewCompleter(c.GlobalString("bash-completion-filter"))
		cmp.Fixed(branches...)
	},
}
