package tracing

import (
	"github.com/SierraSoftworks/sentry-go/v2"
)

var locations = []string{"main"}

func Enter(location string) sentry.Breadcrumb {
	if len(locations) == 0 {
		locations = []string{"main"}
	}

	oldLocation := locations[0]
	locations = append([]string{location}, locations...)

	return sentry.DefaultBreadcrumbs().
			NewNavigation(oldLocation, location).
			WithCategory("app").
			WithLevel(sentry.Info)

}

func Transition(location string) sentry.Breadcrumb {
	if len(locations) == 0 {
		locations = []string{"main"}
	}
	
	oldLocation := locations[0]
	locations[0] = location

	return sentry.DefaultBreadcrumbs().
			NewNavigation(oldLocation, location).
			WithCategory("app").
			WithLevel(sentry.Info)
}

func Exit() sentry.Breadcrumb {
	switch len(locations) {
	case 0:
		locations = []string{"main", "main"}
	case 1:
		locations = append(locations, locations[0])
	default:
	}

	oldLocation = locations[0]
	locations = locations[1:]

	return sentry.DefaultBreadcrumbs().
			NewNavigation(oldLocation, locations[0]).
			WithCategory("app").
			WithLevel(sentry.Info)

}