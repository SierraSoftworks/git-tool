package registry

// A Source is used to provide entries
type Source interface {
	GetEntries() ([]string, error)
	GetEntry(id string) (*Entry, error)
}
