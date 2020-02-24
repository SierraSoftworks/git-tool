package di

import "github.com/SierraSoftworks/git-tool/internal/pkg/registry"

var registrySource registry.Source = registry.GitHub()

func GetRegistry() registry.Source {
	return registrySource
}

func SetRegistry(source registry.Source) {
	registrySource = source
}
