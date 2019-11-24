package registry

// A Source is used to provide entries
type Source interface {
	GetEntries() ([]Entry, error)
}
