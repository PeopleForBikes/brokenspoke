#!/bin/bash
set -euo pipefail

RELEASE_NAME=pgadmin
HOST=$(minikube ip)

# Install chart.
helm upgrade \
  --install \
  -f values.minikube.yaml \
  --set ingress.hosts[0].host="${RELEASE_NAME}.${HOST}.nip.io" \
  --set ingress.hosts[0].paths[0].path="/" \
  --set ingress.hosts[0].paths[0].pathType="Prefix" \
  "${RELEASE_NAME}" \
  runix/pgadmin4

# Wait for the deployment to be complete.
kubectl rollout status "deployment/pgadmin-pgadmin4"
