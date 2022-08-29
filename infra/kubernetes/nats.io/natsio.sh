#!/bin/bash
set -euo pipefail

RELEASE_NAME=nats
NATS_PORT=4222

helm install "${RELEASE_NAME}" nats/nats

# Wait for the deployment to be complete.
kubectl rollout status "statefulset/${RELEASE_NAME}"

echo
echo "To connect from the host to Minikube, start the port forwarding with:"
echo "kubectl port-forward --namespace default svc/${RELEASE_NAME} ${NATS_PORT}:${NATS_PORT}"
echo

