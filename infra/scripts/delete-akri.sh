#!/bin/bash

set -e

echo "Deleting AKRI resources"

kubectl delete -k ../akri/deploy

echo "AKRI resources deleted"