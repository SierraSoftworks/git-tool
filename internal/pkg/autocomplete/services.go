package autocomplete

// Services will generate autocomplete suggestions the services in your config file.
func (c *Completer) Services() {
	for _, svc := range c.Config.GetServices() {
		c.complete(svc.Domain())
	}
}

// ServicePrefixes will generate autocomplete suggestions the services in your config file.
func (c *Completer) ServicePrefixes() {
	for _, svc := range c.Config.GetServices() {
		c.complete(svc.Domain() + "/")
	}
}
