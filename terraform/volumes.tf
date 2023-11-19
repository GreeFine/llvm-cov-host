resource "kubernetes_persistent_volume_claim" "storage" {
  metadata {
    name      = "storage-${local.name}"
    namespace = local.namespace
  }

  spec {
    access_modes = ["ReadWriteOnce"]

    resources {
      requests = {
        storage = "10Gi"
      }
    }
  }
}
