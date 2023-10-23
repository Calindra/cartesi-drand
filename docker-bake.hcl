variable "TAG" {
  default = "latest"
}

group "default" {
  targets = ["middleware"]
}

target "middleware" {
  context = "convenience-middleware"
  dockerfile = "Dockerfile.middleware"
  tags = ["${target.middleware.name}:${TAG}"]
}