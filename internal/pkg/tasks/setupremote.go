package tasks

import (
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/pkg/githosts"
	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/sirupsen/logrus"
)

// SetupRemote is responsible for creating the upstream repository
// on whichever service hosts the repo.
func SetupRemote() Task {
	return &setupRemote{}
}

// setupRemote is responsible for creating the upstream repository
// on whichever service hosts the repo.
type setupRemote struct {
}

// ApplyRepo runs the task against a repository
func (t *setupRemote) ApplyRepo(r models.Repo) error {
	if !di.GetConfig().GetFeatures().CreateRemoteRepo() {
		return nil
	}

	logrus.WithField("repo", r).Debug("Ensuring that remote repository is created")

	host := githosts.GetHost(r.Service())
	if host == nil {
		logrus.WithField("service", r.Service().Domain()).Warning("unable to create remote repository (unsupported service)")
		return nil
	}

	exists, err := host.HasRepo(r)
	if err != nil {
		return err
	}

	if !exists {
		return host.CreateRepo(r)
	}

	return nil
}

// ApplyScratchpad runs the task against a scratchpad
func (t *setupRemote) ApplyScratchpad(r models.Scratchpad) error {
	return nil
}
