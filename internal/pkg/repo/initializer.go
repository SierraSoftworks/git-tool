package repo

import (
	"os"

	"github.com/SierraSoftworks/git-tool/pkg/githosts"

	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/pkg/errors"
	"github.com/sirupsen/logrus"
	"gopkg.in/src-d/go-git.v4"
	"gopkg.in/src-d/go-git.v4/config"
)

type Initializer struct {
}

func (i *Initializer) CreateScratchpad(r models.Repo) error {
	logrus.WithField("repo", r).Debug("Creating new folder for scratchpad")
	if err := os.MkdirAll(r.Path(), os.ModePerm); err != nil {
		return errors.Wrap(err, "repo: unable to create scratchpad directory")
	}

	return nil
}

func (i *Initializer) Init(r models.Repo) error {
	logrus.WithField("repo", r).Debug("Creating new folder for repository")
	if err := os.MkdirAll(r.Path(), os.ModePerm); err != nil {
		return errors.Wrap(err, "repo: unable to create repo directory")
	}

	logrus.WithField("repo", r).Debug("Initializing repository")
	gr, err := git.PlainInit(r.Path(), false)
	if err != nil && err != git.ErrRepositoryAlreadyExists {
		return errors.Wrap(err, "repo: unable to initialize repo")
	}

	if gr == nil {
		gr, err = git.PlainOpen(r.Path())
		if err != nil {
			return errors.Wrap(err, "repo: unable to open repository")
		}
	}

	remote := config.RemoteConfig{
		Name:  "origin",
		URLs:  []string{r.GitURL()},
		Fetch: []config.RefSpec{},
	}

	logrus.WithField("repo", r).Debug("Validating configuration for git remote 'origin'")
	err = remote.Validate()
	if err != nil {
		return errors.Wrap(err, "repo: remote 'origin' is not configured correctly")
	}

	logrus.WithField("repo", r).Debug("Creating git remote 'origin'")
	_, err = gr.CreateRemote(&remote)
	if err != nil && err != git.ErrRemoteExists {
		return errors.Wrap(err, "repo: unable to configure remote 'origin'")
	}

	logrus.WithField("repo", r).Debug("Ensuring that remote repository is created")
	return i.EnsureRemoteRepo(r)
}

func (i *Initializer) Pull(r models.Repo) error {
	gr, err := git.PlainOpen(r.Path())

	if err != nil {
		return errors.Wrap(err, "repo: unable to open repository")
	}

	wt, err := gr.Worktree()
	if err != nil {
		return errors.Wrap(err, "repo: unable to get repository worktree")
	}

	err = wt.Pull(&git.PullOptions{
		RemoteName: "origin",
	})

	if err != nil {
		return errors.Wrap(err, "repo: unable to pull repository")
	}

	return nil
}

func (i *Initializer) Clone(r models.Repo) error {
	_, err := git.PlainClone(r.Path(), false, &git.CloneOptions{
		URL:               r.GitURL(),
		RecurseSubmodules: git.DefaultSubmoduleRecursionDepth,
		Tags:              git.AllTags,
		RemoteName:        "origin",
	})

	if err != nil {
		return errors.Wrap(err, "repo: unable to clone remote repository")
	}

	return nil
}

func (i *Initializer) EnsureRemoteRepo(r models.Repo) error {
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

func (i *Initializer) CreateRemoteRepo(r models.Repo) error {
	host := githosts.GetHost(r.Service())
	if host == nil {
		logrus.WithField("service", r.Service().Domain()).Warning("unable to create remote repository (unsupported service)")
		return nil
	}

	return host.CreateRepo(r)
}
