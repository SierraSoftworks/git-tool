resource "azurerm_static_web_app" "website" {
  name                = "sierrasoftworks-${var.app-name}"
  resource_group_name = azurerm_resource_group.website.name
  location            = local.website_location
  sku_tier            = "Standard"
  sku_size            = "Standard"
  tags                = var.tags

  lifecycle {
    prevent_destroy = true

    ignore_changes = [
      repository_branch,
      repository_url
    ]
  }
}

resource "azurerm_static_web_app_custom_domain" "domain" {
  static_web_app_id = azurerm_static_web_app.website.id
  domain_name       = trimsuffix(azurerm_dns_cname_record.cname.fqdn, ".")
  validation_type   = "cname-delegation"

  lifecycle {
    prevent_destroy = true
  }

  depends_on = [
    azurerm_dns_cname_record.cname
  ]
}
