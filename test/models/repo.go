package testmodels

import (
	"github.com/SierraSoftworks/git-tool/pkg/models"
	"strings"
)

type TestRepo struct {
	ModelFullName string
	ModelPath     string
	ModelService  models.Service

	ModelExists bool
	ModelValid  bool
}

func (r *TestRepo) Name() string {
	parts := strings.Split(r.ModelFullName, "/")
	return parts[len(parts)-1]
}

func (r *TestRepo) Namespace() string {
	parts := strings.Split(r.ModelFullName, "/")
	return strings.Join(parts[:len(parts)-1], "/")
}

func (r *TestRepo) FullName() string {
	return r.ModelFullName
}

func (r *TestRepo) Service() models.Service {
	return r.ModelService
}

func (r *TestRepo) Website() string {
	return r.ModelService.Website(r)
}

func (r *TestRepo) GitURL() string {
	return r.ModelService.GitURL(r)
}

func (r *TestRepo) HttpURL() string {
	return r.ModelService.HttpURL(r)
}

func (r *TestRepo) Path() string {
	return r.ModelPath
}

func (r *TestRepo) Valid() bool {
	return r.ModelValid
}

func (r *TestRepo) Exists() bool {
	return r.ModelExists
}
