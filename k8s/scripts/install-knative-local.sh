#!/usr/bin/env bash
set -euo pipefail
CTX="$(kubectl config current-context 2>/dev/null || echo "")"
if [[ ! "$CTX" =~ ^kind- ]]; then
  echo "install-knative-local: current context '$CTX' is not a local kind cluster, skipping Knative/Kourier install (assumed pre-installed on remote clusters)."
  exit 0
fi
if kubectl get ns knative-serving >/dev/null 2>&1; then
  echo "install-knative-local: knative-serving namespace already present, skipping install."
  exit 0
fi
echo "install-knative-local: installing Knative Serving + Kourier networking layer into local kind cluster '$CTX'..."
# NOTE: pin exact version tags appropriate at time of use; carried over from
# otel-helm/up.sh, which pinned knative-v1.11.0 for serving and used
# net-kourier's "latest" release.
kubectl apply -f https://github.com/knative/serving/releases/download/knative-v1.11.0/serving-crds.yaml
kubectl apply -f https://github.com/knative/serving/releases/download/knative-v1.11.0/serving-core.yaml
kubectl wait --for=condition=Ready pod --all -n knative-serving --timeout=300s

kubectl apply -f https://github.com/knative/net-kourier/releases/latest/download/kourier.yaml
kubectl patch configmap/config-network \
  -n knative-serving \
  --type merge \
  -p '{"data":{"ingress.class":"kourier.ingress.networking.knative.dev"}}'
