data "azuread_client_config" "current" {}

variable "app-name" {
  description = "The name of the static web app to create."
  type        = string
  default     = "git-tool"
}

variable "root-domain" {
  description = "The domain name which will be used for this website."
  type        = string
  default     = "sierrasoftworks.com"
}

variable "location" {
  description = "The location that the static web app should be deployed."
  default     = "North Europe"
}

variable "resource_group" {
  description = "The name of the resource group to deploy into."
  type        = string
  default     = "app-git-tool"
}

variable "tags" {
  description = "The tags which should apply to the resource."
  type        = map(string)
}

variable "location_override" {
  description = "The location into which the website itself should be deployed, if different to the default."
  type        = string
  default     = "West Europe"
}

locals {
  website_location = var.location_override != "" ? var.location_override : var.location
}
