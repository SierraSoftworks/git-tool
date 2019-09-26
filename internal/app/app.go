package app

import (
	"fmt"
	"strings"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/repo"
	"github.com/SierraSoftworks/git-tool/internal/pkg/tracing"
	"github.com/sirupsen/logrus"
	"github.com/urfave/cli"
)

func init() {
	di.SetConfig(config.Default())
	di.SetMapper(&repo.Mapper{})
	di.SetInitializer(&repo.Initializer{})
}

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
		openScratchCommand,
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
		tracing.Enter("/app/before")
		defer tracing.Exit()

		if c.GlobalString("config") != "" {
			tracing.Transition("/app/before/loadConfig")
			logrus.WithField("config_path", c.GlobalString("config")).Debug("Loading configuration file")
			cfgResult, err := config.Load(c.GlobalString("config"))
			if err != nil {
				return err
			}

			logrus.WithField("config_path", c.GlobalString("config")).Debug("Loaded configuration file")
			di.SetConfig(cfgResult)
		}

		if c.GlobalBool("verbose") {
			logrus.SetLevel(logrus.DebugLevel)
		}

		return nil
	}

	app.BashComplete = func(c *cli.Context) {
		tracing.Enter("/app/complete/")
		defer tracing.Exit()

		filter := c.GlobalString("bash-completion-filter")

		for _, cmd := range c.App.Commands {
			for _, name := range cmd.Names() {
				if filter == "" || strings.HasPrefix(strings.ToLower(name), strings.ToLower(filter)) {
					fmt.Fprintln(di.GetOutput(), name)
				}
			}
		}
	}

	app.Writer = di.GetOutput()
	app.ErrWriter = di.GetOutput()

	return app
}
