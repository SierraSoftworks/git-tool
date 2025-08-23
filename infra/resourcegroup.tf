resource "azurerm_resource_group" "website" {
  name     = var.resource_group
  location = var.location
  tags = merge(var.tags, {
    domain = "${var.app-name}.${var.root-domain}"
  })

  lifecycle {
    prevent_destroy = true
  }
}
