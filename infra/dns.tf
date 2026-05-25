data "cloudflare_zones" "root_domain" {
  account = {
    id = var.cloudflare_account_id
  }

  name = var.root-domain
}

resource "cloudflare_dns_record" "cname" {
  zone_id = data.cloudflare_zones.root_domain.result[0].id
  name    = var.app-name
  content = "sierrasoftworks.github.io"
  type    = "CNAME"
  ttl     = 1
  proxied = true
}
