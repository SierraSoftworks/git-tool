package app

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/tracing"
	"github.com/SierraSoftworks/update-go"
	"github.com/urfave/cli"
)

var updateManager update.Manager

var updateCommand = cli.Command{
	Name:  "update",
	Usage: "Updates git-tool to the latest available version.",
	Flags: []cli.Flag{
		cli.BoolFlag{
			Name: "list",
		},
	},
	UsageText: "[VERSION]",
	Action: func(c *cli.Context) error {
		tracing.Enter("/app/command/update")
		defer tracing.Exit()

		releases, err := updateManager.Source.Releases()
		if err != nil {
			return err
		}

		if c.Bool("list") {
			for _, release := range releases {
				listStyle := "-"
				if release.ID == c.App.Version {
					listStyle = "*"
				}

				fmt.Fprintf(di.GetOutput(), " %s %s\n", listStyle, release.ID)
			}
			return nil
		}

		var targetRelease *update.Release

		if c.NArg() > 0 {
			for _, r := range releases {
				if r.ID == c.Args().First() {
					targetRelease = &r
					break
				}
			}

			if targetRelease == nil {
				return fmt.Errorf("usage: could not find update with provided tag")
			}
		} else {
			currentVersion := c.App.Version
			if strings.HasPrefix(currentVersion, "v") {
				currentVersion = currentVersion[1:]
			}

			targetRelease = update.LatestUpdate(releases, currentVersion)
			if targetRelease == nil {
				fmt.Fprintf(di.GetOutput(), "No update available\n")
				return nil
			}
		}

		ctx, cancel := context.WithTimeout(context.Background(), 120*time.Second)
		defer cancel()

		tracing.Transition("/app/command/updating").WithMessage(fmt.Sprintf("Updating to %s", targetRelease.ID))

		fmt.Fprintf(di.GetOutput(), "Downloading update %s...\n", targetRelease.ID)

		return updateManager.Update(ctx, targetRelease)
	},
}
