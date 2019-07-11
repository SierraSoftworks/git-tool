package repo

import (
	"github.com/SierraSoftworks/git-tool/pkg/models"
	"os"
	"strings"
)

// scratchpad describes a repository object with any necessary properties required by
// Git-Tool.
type scratchpad struct {
	fullName string
	path     string
}

func (r *scratchpad) FullName() string {
	return r.fullName
}

func (r *scratchpad) Namespace() string {
	parts := strings.Split(r.fullName, "/")
	return strings.Join(parts[:len(parts)-1], "/")
}

func (r *scratchpad) Name() string {
	parts := strings.Split(r.fullName, "/")
	return parts[len(parts)-1]
}

func (r *scratchpad) Service() models.Service {
	return &scratchpadService{}
}

func (r *scratchpad) Path() string {
	return r.path
}

func (r *scratchpad) Website() string {
	return ""
}

func (r *scratchpad) HttpURL() string {
	return ""
}

func (r *scratchpad) GitURL() string {
	return ""
}

func (r *scratchpad) Exists() bool {
	s, err := os.Stat(r.path)
	if err != nil && os.IsNotExist(err) {
		return false
	}

	if err != nil {
		return true
	}

	return s.IsDir()
}

func (r *scratchpad) Valid() bool {
	return true
}

type scratchpadService struct {}

func (s *scratchpadService) Domain() string {
	return "scratch"
}

func (s *scratchpadService) DirectoryGlob() string {
	return "*"
}

func (s *scratchpadService) Website(r models.Repo) string {
	return ""
}

func (s *scratchpadService) GitURL(r models.Repo) string {
	return ""
}

func (s *scratchpadService) HttpURL(r models.Repo) string {
	return ""
}