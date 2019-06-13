package autocomplete

// Apps will generate autocomplete suggestions for applications in your config file.
func (c *Completer) Apps() {
	for _, app := range c.Config.GetApps() {
		c.complete(app.Name())
	}
}
