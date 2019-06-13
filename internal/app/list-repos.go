package app

import (
	"fmt"
	"strings"

	"github.com/SierraSoftworks/git-tool/internal/pkg/templates"

	"github.com/urfave/cli"
)

var listReposCommand = cli.Command{
	Name: "list",
	Aliases: []string{
		"ls",
		"ll",
	},
	Usage:     "Lists the repositories in your local development environment.",
	ArgsUsage: "[filter]",
	Flags: []cli.Flag{
		cli.BoolFlag{
			Name:  "quiet,q",
			Usage: "show only the fully qualified repository names",
		},

		cli.BoolFlag{
			Name:  "full",
			Usage: "show all available information about each repository",
		},
	},
	Action: func(c *cli.Context) error {
		repos, err := getMapper().GetRepos()
		if err != nil {
			return err
		}

		filter := c.Args().First()

		for i, repo := range repos {
			if filter != "" && !strings.Contains(templates.RepoQualifiedName(repo), filter) {
				continue
			}

			if c.Bool("quiet") {
				fmt.Println(templates.RepoQualifiedName(repo))
			} else if c.Bool("full") {
				if i > 0 {
					fmt.Println("---")
				}

				fmt.Println(templates.RepoFullInfo(repo))
			} else {
				fmt.Println(templates.RepoShortInfo(repo))
			}
		}

		return nil
	},
}
