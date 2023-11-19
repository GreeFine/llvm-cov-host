variable "API_KEY" {
  description = "Global API KEY"
  type        = string
  default     = ""
}

locals {
  key_path = "/keys"
}

resource "kubernetes_secret" "env" {
  lifecycle {
    # ignore_changes = all
  }

  metadata {
    name      = "${local.name}-env"
    namespace = local.namespace
  }

  data = {
    API_KEY      = var.API_KEY
    SSH_KEY_PATH = "${local.key_path}/id_ed25519"
    RUST_LOG     = "info"
  }

  immutable = true
  type      = "Opaque"
}

resource "kubernetes_secret" "keys" {
  lifecycle {
    ignore_changes = all
  }

  metadata {
    name      = "${local.name}-keys"
    namespace = local.namespace
  }

  data = {
    ssh_ed25519_key       = fileexists("id_ed25519") ? file("id_ed25519") : ""
    "ssh_ed25519_key.pub" = fileexists("id_ed25519.pub") ? file("id_ed25519") : ""
  }

  immutable = true
  type      = "Opaque"
}
