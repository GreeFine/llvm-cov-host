resource "kubernetes_service" "llvm_cov_host" {
  metadata {
    name      = local.name
    namespace = local.namespace
  }

  spec {
    port {
      name = "http"
      port = 8080
    }

    selector = {
      run = local.name
    }

    type = "ClusterIP"
  }
}

resource "kubernetes_ingress_v1" "llvm_cov_host" {
  metadata {
    name      = local.name
    namespace = local.namespace

    annotations = {
      "kubernetes.io/ingress.class"                           = "traefik"
      "traefik.ingress.kubernetes.io/router.tls"              = "true"
      "traefik.ingress.kubernetes.io/router.tls.certresolver" = "letsencrypt"
      # "traefik.ingress.kubernetes.io/router.middlewares"      = "traefik-wireguard-ip-whitelist@kubernetescrd"  
    }
  }

  spec {
    rule {
      host = "coverage.preview.blackfoot.dev"

      http {
        path {
          backend {
            service {
              name = kubernetes_service.llvm_cov_host.metadata[0].name
              port {
                name = kubernetes_service.llvm_cov_host.spec[0].port[0].name
              }
            }

          }
        }
      }
    }
  }
}
