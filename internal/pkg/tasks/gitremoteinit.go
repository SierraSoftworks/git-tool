package tasks

import (
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/pkg/errors"
	"github.com/sirupsen/logrus"
	"gopkg.in/src-d/go-git.v4"
	"gopkg.in/src-d/go-git.v4/config"
)

// GitRemote is responsible for running the equivalent of a `git remote add` or
// `git remote set-url` for a repository.
func GitRemote(remoteName string) Task {
	return &gitRemote{
		RemoteName: remoteName,
	}
}

// gitRemote is responsible for running the equivalent of a `git remote add` or
// `git remote set-url` for a repository.
type gitRemote struct {
	RemoteName string
}

// ApplyRepo runs the task against a repository
func (t *gitRemote) ApplyRepo(r models.Repo) error {
	gr, err := git.PlainOpen(r.Path())
	if err != nil {
		return errors.Wrap(err, "repo: unable to open git repository")
	}

	url := r.GitURL()
	if di.GetConfig().GetFeatures().UseHttpTransport() {
		url = r.HttpURL()
	}

	remoteName := t.RemoteName
	if remoteName == "" {
		remoteName = "origin"
	}

	remote := config.RemoteConfig{
		Name:  remoteName,
		URLs:  []string{url},
		Fetch: []config.RefSpec{},
	}

	logrus.WithField("repo", r).Debugf("Validating configuration for git remote '%s'", remoteName)
	err = remote.Validate()
	if err != nil {
		return errors.Wrap(err, "repo: remote 'origin' is not configured correctly")
	}

	gr.DeleteRemote(remoteName)

	logrus.WithField("repo", r).Debugf("Creating git remote '%s'", remoteName)
	_, err = gr.CreateRemote(&remote)
	if err != nil && err != git.ErrRemoteExists {
		return errors.Wrapf(err, "repo: unable to configure remote '%s'", remoteName)
	}

	return nil
}

// ApplyScratchpad runs the task against a scratchpad
func (t *gitRemote) ApplyScratchpad(r models.Scratchpad) error {
	return nil
}
