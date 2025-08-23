resource "azurerm_dns_cname_record" "cname" {
  name                = var.app-name
  resource_group_name = "dns"
  zone_name           = var.root-domain
  ttl                 = 300
  target_resource_id  = azurerm_static_web_app.website.id

  lifecycle {
    prevent_destroy = true
  }

  depends_on = [
    azurerm_static_web_app.website
  ]
}
