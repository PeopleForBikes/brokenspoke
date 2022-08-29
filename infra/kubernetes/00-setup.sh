#!/bin/bash
set -euo pipefail

# TODO: Pin Brew versions for Mac
# TODO: Figure out a better mechanism for pinning versions in general
#       There are multiple ways to validate signatures, checksums, etc.

# PINNED VERSIONS GO HERE
MINIKUBE_VERSION=v1.25.2
MINIKUBE_FILE_NAME=minikube-linux-amd64
MINIKUBE_URL=https://github.com/kubernetes/minikube/releases/download/$MINIKUBE_VERSION/$MINIKUBE_FILE_NAME
MINIKUBE_SHA256="ef610fa83571920f1b6c8538bb31a8dc5e10ff7e1fcdca071b2a8544c349c6fd"

HELM_VERSION=v3.8.2
HELM_FILE_NAME=helm-${HELM_VERSION}-linux-amd64.tar.gz
HELM_URL=https://get.helm.sh/$HELM_FILE_NAME
HELM_SHA256="6cb9a48f72ab9ddfecab88d264c2f6508ab3cd42d9c09666be16a7bf006bed7b"

KUBECTL_VERSION=v1.22.3
KUBECTL_FILE_NAME=kubectl
KUBECTL_URL=https://dl.k8s.io/release/$KUBECTL_VERSION/bin/linux/amd64/kubectl
KUBECTL_VALIDATE_CHECKSUM_URL=$KUBECTL_URL.sha256

# Define variables.
C_GREEN='\033[32m'
C_YELLOW='\033[33m'
C_RED='\033[31m'
C_RESET_ALL='\033[0m'

# Detect the platform.
PLATFORM=$(uname)

# Install packages if needed.
echo -e "${C_GREEN}Installing packages if needed...${C_RESET_ALL}"
case "${PLATFORM}" in

  Darwin)
    minikube version || brew install minikube
    helm version || brew install helm
    kubectl version --client || brew install kubectl
    jq --version || brew install jq
    ;;

  Linux)
    [[ $(minikube version | awk '{print $3}' | xargs) == "$MINIKUBE_VERSION" ]] || (
      echo -e "${C_GREEN}minikube not found, installing...${C_RESET_ALL}"
      TMP=$(mktemp -d)
      pushd "$TMP"
      curl -LO $MINIKUBE_URL
      ACTUAL_SHA256=$(sha256sum $MINIKUBE_FILE_NAME | awk '{print $1}')
      [[ $ACTUAL_SHA256 == "$MINIKUBE_SHA256" ]] || (
        echo "Expected SHA256 for $MINIKUBE_FILE_NAME: $MINIKUBE_SHA256"
        echo "Actual SHA256 for $MINIKUBE_FILE_NAME: $ACTUAL_SHA256"
        exit 1
      )
      sudo install $MINIKUBE_FILE_NAME $INSTALL_DIR/minikube
      rm $MINIKUBE_FILE_NAME
      popd
      rmdir "$TMP"
    )

    [[ $(helm version | awk '{print $1 }' | sed -r 's/.*Version:\"(.*)\",/\1/') == "$HELM_VERSION" ]] || (
      echo -e "${C_GREEN}helm not found, installing...${C_RESET_ALL}"
      TMP=$(mktemp -d)
      pushd "$TMP"
      curl -LO $HELM_URL
      ACTUAL_SHA256=$(sha256sum $HELM_FILE_NAME | awk '{print $1}')
      [[ $ACTUAL_SHA256 == "$HELM_SHA256" ]] || (
        echo "Expected SHA256 for $HELM_FILE_NAME: $HELM_SHA256"
        echo "Actual SHA256 for $HELM_FILE_NAME: $ACTUAL_SHA256"
        exit 1
      )
      tar xvf $HELM_FILE_NAME
      sudo install linux-amd64/helm $INSTALL_DIR/helm
      rm -rf linux-amd64
      rm $HELM_FILE_NAME
      popd
      rmdir "$TMP"
    )

    kubectl version --client || (
      echo -e "${C_GREEN}kubectl not found, installing...${C_RESET_ALL}"
      TMP=$(mktemp -d)
      pushd "$TMP"
      curl -LO $KUBECTL_URL
      curl -LO $KUBECTL_VALIDATE_CHECKSUM_URL
      echo "$(<kubectl.sha256) kubectl" | sha256sum --check
      sudo install kubectl $INSTALL_DIR/kubectl
      rm $KUBECTL_FILE_NAME
      rm $KUBECTL_FILE_NAME.sha256
      popd
      rmdir "$TMP"
    )
    ;;

  *)
    echo -e "${C_RED}The ${PLATFORM} platform is unimplemented or unsupported.${C_RESET_ALL}"
    exit 1
    ;;

esac

# Start the service.
# shellcheck disable=SC1083
MINIKUBE_STATUS=$(minikube status --format  {{.Host}} || true)
if [ "${MINIKUBE_STATUS}" == "Running" ]; then
  echo -e "${C_YELLOW}Minikube is already running.${C_RESET_ALL}"
else
  echo -e "${C_GREEN}Starting Minikube...${C_RESET_ALL}"
  minikube start \
    --driver=docker \
    --memory max
fi

# Set up Minikube context.
echo -e "${C_GREEN}Configuring minikube context...${C_RESET_ALL}"
kubectl config use-context minikube

# Display a message to tell to update the environment variables.
minikube docker-env

# Manage default Ingress Controller.
minikube addons enable ingress

# Add/Update Helm chart repositories.
echo -e "${C_GREEN}Configuring helm...${C_RESET_ALL}"
helm repo add stable https://charts.helm.sh/stable
helm repo add bitnami https://charts.bitnami.com/bitnami
helm repo add runix https://helm.runix.net
helm repo add nats https://nats-io.github.io/k8s/helm/charts
helm repo update
