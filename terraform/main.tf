terraform {
  required_providers {
    kubernetes = {
      source = "hashicorp/kubernetes"
    }
  }
  backend "kubernetes" {
    secret_suffix  = "state"
    config_path    = "~/.kube/config"
    config_context = "bf-dev" # Replace this with your own context
    # WARNING: We use terraform workspace to manage the different environment, but the secret file must be the same
    namespace = "llvm-cov-host"
  }
}

provider "kubernetes" {
  config_path    = "~/.kube/config"
  config_context = "bf-dev" # Replace this with your own context
}

locals {
  name      = "llvm-cov-host"
  namespace = "llvm-cov-host"
}

variable "IMAGE_TAG" {
  type = string
}

variable "REGISTRY_PATH" {
  type = string
}
