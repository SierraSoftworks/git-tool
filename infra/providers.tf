terraform {
  required_version = ">= 1.1.0"

  required_providers {
    honeycombio = {
      source  = "honeycombio/honeycombio"
      version = "~> 0.50.0"
    }

    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 5.0"
    }
  }
}

terraform {
  cloud {
    organization = "sierrasoftworks"

    workspaces {
      name = "git-tool"
    }
  }
}

provider "cloudflare" {}
