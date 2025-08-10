#!/bin/bash

echo "=== Setting up local Knative environment with Kind ==="
echo

# Check if kind is installed
if ! command -v kind &> /dev/null; then
    echo "Installing kind..."
    # For macOS with Homebrew
    if command -v brew &> /dev/null; then
        brew install kind
    else
        echo "Please install kind manually: https://kind.sigs.k8s.io/docs/user/quick-start/#installation"
        exit 1
    fi
fi

# Check if kubectl is installed
if ! command -v kubectl &> /dev/null; then
    echo "Installing kubectl..."
    if command -v brew &> /dev/null; then
        brew install kubectl
    else
        echo "Please install kubectl manually"
        exit 1
    fi
fi

echo "1. Creating Kind cluster..."
kind create cluster --name knative-local --config kind-config.yaml

echo "2. Installing Knative Serving..."
kubectl apply -f https://github.com/knative/serving/releases/download/knative-v1.11.0/serving-crds.yaml
kubectl apply -f https://github.com/knative/serving/releases/download/knative-v1.11.0/serving-core.yaml

echo "4. Waiting for Knative to be ready..."
kubectl wait --for=condition=Ready pod --all -n knative-serving --timeout=30s

kubectl apply -f https://github.com/knative/net-kourier/releases/latest/download/kourier.yaml
kubectl patch configmap/config-network \
  -n knative-serving \
  --type merge \
  -p '{"data":{"ingress.class":"kourier.ingress.networking.knative.dev"}}'

echo "5. Getting Kourier service info..."
kubectl get svc kourier -n kourier-system