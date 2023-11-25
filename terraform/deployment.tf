resource "kubernetes_deployment" "llvm_cov_host" {
  depends_on = [kubernetes_secret.env]
  metadata {
    name      = local.name
    namespace = local.namespace
  }

  spec {
    strategy {
      type = "Recreate"
    }
    selector {
      match_labels = {
        run = local.name
      }
    }

    template {
      metadata {
        labels = {
          run = local.name
        }
      }

      spec {
        container {
          name  = local.name
          image = "${var.REGISTRY_PATH}/llvm-cov-host:${var.IMAGE_TAG}"

          port {
            container_port = 8080
          }

          env_from {
            secret_ref {
              name = kubernetes_secret.env.metadata[0].name
            }
          }

          volume_mount {
            name       = "storage"
            mount_path = "/app/output"
          }
          volume_mount {
            name       = "keys"
            mount_path = local.key_path
            read_only  = true
          }
          resources {
            limits = {
              cpu    = "1100m"
              memory = "1000Mi"
            }
            requests = {
              cpu    = "50m"
              memory = "10Mi"
            }
          }
        }


        volume {
          name = "storage"
          persistent_volume_claim {
            claim_name = kubernetes_persistent_volume_claim.storage.metadata[0].name
          }
        }
        volume {
          name = "keys"
          secret {
            secret_name  = kubernetes_secret.keys.metadata[0].name
            default_mode = "0600"
          }
        }
      }
    }
  }
}
