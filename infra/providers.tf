terraform {
  required_version = ">= 1.1.0"

  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 4.46.0"
    }

    azuread = {
      source  = "hashicorp/azuread"
      version = "~> 3.6.0"
    }

    honeycombio = {
      source  = "honeycombio/honeycombio"
      version = "~> 0.41.0"
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

provider "azurerm" {
  features {}

  // NOTE: You can retrieve this secret using `op read op://epfkgzb2bz4msye2xrhffiz3se/jrlwg64m56hkbkbfvgljfkwcfy/Azure/client_secret`
  subscription_id = var.azure_subscription
  tenant_id       = var.azure_tenant
}

provider "azuread" {
  // NOTE: You can retrieve this secret using `op read op://epfkgzb2bz4msye2xrhffiz3se/jrlwg64m56hkbkbfvgljfkwcfy/Azure/client_secret`
  tenant_id = var.azure_tenant
}



variable "azure_subscription" {

}

variable "azure_tenant" {

}
