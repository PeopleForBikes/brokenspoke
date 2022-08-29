#!/bin/bash
set -euo pipefail

# Set variables.
PGUSER=minikube
PGPASSWORD=minikube
PGHOST=127.0.0.1
PGPORT=5432
PGDATABASE=bna

# Install postgresql.
helm upgrade \
  --install \
  -f values.minikube.yaml \
  --set auth.username=${PGUSER}\
  --set auth.password=${PGPASSWORD} \
  --set auth.database=${PGDATABASE} \
  postgresql \
  bitnami/postgresql

# Wait for the deployment to be complete.
kubectl rollout status statefulset/postgresql

echo "Connection string:"
echo "export DATABASE_URL=postgresql://${PGUSER}:${PGPASSWORD}@${PGHOST}:${PGPORT}/${PGDATABASE}"
echo
