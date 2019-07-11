package repo

import (
	"io/ioutil"
	"os"
	"path/filepath"
	"strings"

	"github.com/SierraSoftworks/git-tool/internal/pkg/autocomplete"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/templates"
	"github.com/SierraSoftworks/git-tool/pkg/models"

	"github.com/sirupsen/logrus"

	"github.com/pkg/errors"
)

// A Mapper holds the information about a developer's source code folder which
// may contain multiple repositories.
type Mapper struct {
}

// GetBestRepo gets the repo which best matches a given name
func (d *Mapper) GetBestRepo(name string) (models.Repo, error) {
	if a := di.GetConfig().GetAlias(name); a != "" {
		name = a
	}

	r, err := d.GetRepo(name)
	if err != nil {
		return r, err
	}

	if r != nil {
		return r, nil
	}

	rs, err := d.GetRepos()
	if err != nil {
		return nil, err
	}

	matched := []models.Repo{}

	for _, rr := range rs {
		if autocomplete.Matches(templates.RepoQualifiedName(rr), name) {
			matched = append(matched, rr)
		}
	}

	if len(matched) == 1 {
		return matched[0], nil
	}

	return nil, errors.New("could not find repository")
}

// GetRepos will fetch all of the repositories contained within a developer's dev
// directory which match the required naming scheme.
func (d *Mapper) GetRepos() ([]models.Repo, error) {
	logrus.WithField("path", di.GetConfig().DevelopmentDirectory()).Debug("Searching for repositories")

	files, err := ioutil.ReadDir(di.GetConfig().DevelopmentDirectory())
	if err != nil {
		return nil, errors.Wrapf(err, "repo: unable to list directory contents in dev directory '%s'", di.GetConfig().DevelopmentDirectory())
	}

	repos := []models.Repo{}

	for _, f := range files {
		if !f.IsDir() {
			continue
		}

		service := di.GetConfig().GetService(f.Name())
		if service == nil {
			logrus.WithField("service", f.Name()).Warn("Could not find a matching service entry in your configuration")
			continue
		}

		childRepos, err := d.GetReposForService(service)
		if err != nil {
			return nil, errors.Wrapf(err, "repo: unable to list directory contents in service directory '%s'", di.GetConfig().DevelopmentDirectory())
		}

		repos = append(repos, childRepos...)
	}

	return repos, nil
}

// GetScratchpads will fetch all of the known scratchpads which are stored locally.
func (d *Mapper) GetScratchpads() ([]models.Repo, error) {
	return d.GetReposForService(&scratchpadService{})
}

// GetScratchpad will fetch a scratchpad repo with the provided name
func (d *Mapper) GetScratchpad(name string) (models.Repo, error) {
	return d.GetRepoForService(&scratchpadService{}, name)
}

// EnsureRepo will ensure that a repository directory has been created at the correct location
// on the filesystem.
func (d *Mapper) EnsureRepo(service models.Service, r models.Repo) error {
	path := filepath.Join(di.GetConfig().DevelopmentDirectory(), service.Domain(), filepath.FromSlash(r.FullName()))

	s, err := os.Stat(path)
	if err != nil {
		if os.IsNotExist(err) {
			if err := os.MkdirAll(path, os.ModePerm); err != nil {
				return errors.Wrapf(err, "repo: unable to create repository directory '%s'", path)
			}
			return nil
		}

		return errors.Wrapf(err, "repo: unable to check directory '%s'", path)
	}

	if s.IsDir() {
		return nil
	}

	return errors.Errorf("repo: repository name already exists and is not a directory '%s'", path)
}

// GetReposForService will fetch all of the known repositories for a specific service.
func (d *Mapper) GetReposForService(service models.Service) ([]models.Repo, error) {
	logrus.WithField("service", service.Domain()).Debug("Enumerating repositories for service")

	path := filepath.Join(di.GetConfig().DevelopmentDirectory(), service.Domain())

	pattern := filepath.Join(path, service.DirectoryGlob())

	files, err := filepath.Glob(pattern)
	if err != nil {
		return nil, errors.Wrapf(err, "repo: unable to list directory contents in service directory '%s'", pattern)
	}

	repos := []models.Repo{}
	for _, f := range files {
		logrus.WithField("service", service.Domain()).WithField("path", f).Debug("Enumerated possible repository")
		r := &repo{
			service:  service,
			fullName: strings.Trim(strings.Replace(f[len(path):], string(filepath.Separator), "/", -1), "/"),
			path:     f,
		}

		if r.Exists() {
			repos = append(repos, r)
		} else {
			logrus.WithField("service", service.Domain()).WithField("path", f).Debug("Marked repository as invalid")
		}
	}

	return repos, nil
}

// GetRepo attempts to resolve the details of a repository given its name.
func (d *Mapper) GetRepo(name string) (models.Repo, error) {
	if name == "" {
		return d.GetCurrentDirectoryRepo()
	}

	dirParts := strings.Split(filepath.ToSlash(name), "/")
	if len(dirParts) < 2 {
		logrus.WithField("path", name).Debug("Not a fully qualified repository name")
		return nil, nil
	}

	serviceName := dirParts[0]
	service := di.GetConfig().GetService(serviceName)

	if service != nil {
		r, err := d.GetRepoForService(service, filepath.Join(dirParts[1:]...))
		return r, err
	}

	r, err := d.GetFullyQualifiedRepo(name)
	if err != nil {
		return r, err
	}

	if r == nil {
		r, err = d.GetRepoForService(di.GetConfig().GetDefaultService(), name)
		if r != nil {
			return r, err
		}
	}

	logrus.WithField("path", name).Debug("Could not find a matching repository")
	return nil, nil
}

// GetRepoForService fetches the repo details for the named repository managed by the
// provided service.
func (d *Mapper) GetRepoForService(service models.Service, name string) (models.Repo, error) {
	dirParts := strings.Split(filepath.ToSlash(name), "/")

	fullNameLength := len(strings.Split(service.DirectoryGlob(), "/"))
	if len(dirParts) < fullNameLength {
		logrus.WithField("path", name).Debug("Not a fully named repository folder within the service's development directory")
		return nil, nil
	}

	return &repo{
		fullName: strings.Join(dirParts[:fullNameLength], "/"),
		service:  service,
		path:     filepath.Join(di.GetConfig().DevelopmentDirectory(), service.Domain(), filepath.Join(dirParts[:fullNameLength]...)),
	}, nil
}

// GetFullyQualifiedRepo fetches the repo details for the fully qualified named
// repository which has been provided.
func (d *Mapper) GetFullyQualifiedRepo(name string) (models.Repo, error) {
	dirParts := strings.Split(filepath.ToSlash(name), "/")

	if len(dirParts) < 2 {
		// Not within a service's repository
		logrus.WithField("path", name).Debug("Not a repository folder within the development directory")
		return nil, nil
	}

	serviceName := dirParts[0]
	service := di.GetConfig().GetService(serviceName)
	if service == nil {
		logrus.WithField("path", name).Debug("No service found to handle repository type")
		return nil, nil
	}

	return d.GetRepoForService(service, strings.Join(dirParts[1:], "/"))
}

// GetCurrentDirectoryRepo gets the repo details for the repository open in your
// current directory.
func (d *Mapper) GetCurrentDirectoryRepo() (models.Repo, error) {
	dir, err := os.Getwd()
	if err != nil {
		return nil, errors.Wrap(err, "repo: failed to get current directory")
	}

	if !strings.HasPrefix(dir, di.GetConfig().DevelopmentDirectory()) {
		logrus.WithField("path", dir).Debug("Not within the development directory")
		return nil, nil
	}

	localDir := strings.Trim(filepath.ToSlash(dir[len(di.GetConfig().DevelopmentDirectory()):]), "/")
	return d.GetFullyQualifiedRepo(localDir)
}
