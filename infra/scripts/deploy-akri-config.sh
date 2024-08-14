#!/bin/bash

set -e
set -u
set -o pipefail

# Verify ACR_NAME is set
if [ -z "$ACR_NAME" ]; then
    echo "ACR_NAME is not set"
    exit 1
fi

# Verify kubectl is installed
if ! command -v kubectl &> /dev/null; then
    echo "kubectl could not be found"
    exit 1
fi

navigate_to_scripts_dir() {
  SCRIPT_DIR=$(dirname $(dirname "$0"))
  cd "$SCRIPT_DIR" || exit
}

navigate_to_scripts_dir

pushd ../akri/deploy || exit

# Replace the placeholder ACR name in device-broker-config.yaml with the actual ACR name
sed "s/replaceme/$ACR_NAME/g" device-broker-config.yaml.template > device-broker-config.yaml

# Deploy device broker 
kubectl apply -f device-broker-config.yaml

# Replace the placeholder ACR name in discovery-handler-config.yaml with the actual ACR name
sed "s/replaceme/$ACR_NAME/g" discovery-handler-config.yaml.template > discovery-handler-config.yaml

# Deploy akri components and discovery handler
kubectl apply -k .
echo "Deployed Akri Config"

popd
