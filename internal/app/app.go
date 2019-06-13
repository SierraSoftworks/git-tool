package app

import (
	"fmt"
	"strings"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/sirupsen/logrus"
	"github.com/urfave/cli"
)

var cfg = config.Default()

// NewApp creates a new command line application for Git-Tool
func NewApp() *cli.App {
	app := cli.NewApp()

	app.Name = "gt"
	app.Author = "Benjamin Pannell <benjamin@pannell.dev>"
	app.Copyright = "Copyright © Sierra Softworks 2019"
	app.Usage = "Manage your git repositories"
	app.Version = "0.0.0-dev"

	app.EnableBashCompletion = true

	app.Description = "A tool which helps manage your local git repositories and development folders."

	app.Commands = []cli.Command{
		repoInfoCommand,
		openAppCommand,
		newRepoCommand,
		listReposCommand,
		listAppsCommand,
		listServicesCommand,
		getGitignoreCommand,
		completeCommand,
	}

	app.Flags = []cli.Flag{
		cli.StringFlag{
			Name:   "config,c",
			EnvVar: "GITTOOL_CONFIG",
			Usage:  "specify the path to your configuration file",
		},
		cli.BoolFlag{
			Name:  "verbose",
			Usage: "enable verbose logging",
		},
		cli.StringFlag{
			Name:   "bash-completion-filter",
			Usage:  "A filter used to select matches for the local argument",
			Hidden: true,
		},
	}

	app.Before = func(c *cli.Context) error {
		if c.GlobalString("config") != "" {
			logrus.WithField("config_path", c.GlobalString("config")).Debug("Loading configuration file")
			cfgResult, err := config.Load(c.GlobalString("config"))
			if err != nil {
				return err
			}

			logrus.WithField("config_path", c.GlobalString("config")).Debug("Loaded configuration file")
			cfg = cfgResult
		}

		if c.GlobalBool("verbose") {
			logrus.SetLevel(logrus.DebugLevel)
		}

		return nil
	}

	app.BashComplete = func(c *cli.Context) {
		filter := c.GlobalString("bash-completion-filter")

		for _, cmd := range c.App.Commands {
			for _, name := range cmd.Names() {
				if filter == "" || strings.HasPrefix(strings.ToLower(name), strings.ToLower(filter)) {
					fmt.Println(name)
				}
			}
		}
	}

	return app
}
