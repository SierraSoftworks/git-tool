package repo

import (
	"github.com/SierraSoftworks/git-tool/pkg/models"
)

type ServiceRegistry interface {
	GetService(name string) models.Service
	GetDefaultService() models.Service
}