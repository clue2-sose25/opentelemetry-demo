#!/bin/bash

echo "=== Complete Knative Local Test Workflow ==="
echo

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
echo "Checking prerequisites..."
if ! command_exists docker; then
    echo "❌ Docker is required. Please install Docker first."
    exit 1
fi

if ! command_exists kubectl; then
    echo "❌ kubectl is required. Please install kubectl first."
    exit 1
fi

if ! command_exists kind; then
    echo "⚠️  Kind not found. Installing..."
    if command_exists brew; then
        brew install kind
    else
        echo "Please install kind manually: https://kind.sigs.k8s.io/docs/user/quick-start/"
        exit 1
    fi
fi

echo "✅ All prerequisites met!"
echo

# Step 1: Setup Kind cluster with Knative
echo "🚀 Step 1: Setting up Kind cluster with Knative..."
if kind get clusters | grep -q "knative-local"; then
    echo "📋 Kind cluster 'knative-local' already exists. Using existing cluster."
else
    ./setup-local-knative.sh
fi

# Step 2: Build the application
echo
echo "🔨 Step 2: Building the application..."
mvn clean package -DskipTests

# Step 3: Build Docker image
echo
echo "🐳 Step 3: Building Docker image..."
docker build -t tawalaya/quote-service-knative:latest .

# Step 4: Load image into Kind
echo
echo "📦 Step 4: Loading image into Kind cluster..."
kind load docker-image tawalaya/quote-service-knative:latest --name knative-local --nodes knative-local-control-plane

# Step 5: Deploy to Knative
echo
echo "🚀 Step 5: Deploying to Knative..."
kubectl apply -f knative-service.yaml

# Step 6: Wait for deployment
echo
echo "⏳ Step 6: Waiting for service to be ready..."
kubectl wait --for=condition=Ready ksvc/quoteservice --timeout=300s

# Step 7: Get service URL and test
echo
echo "🧪 Step 7: Testing the deployed service..."
SERVICE_URL=$(kubectl get ksvc quoteservice -o jsonpath='{.status.url}')
echo "Service URL: $SERVICE_URL"

# run a curl job to test the service
echo "Testing the service with curl..."
kubectl run curl-job --image=curlimages/curl --restart=Never --rm -i --tty -- /bin/sh -c "curl -X POST -s $SERVICE_URL/getquote -H 'Content-Type: application/json' -d '{\"numberOfItems\":3}'"