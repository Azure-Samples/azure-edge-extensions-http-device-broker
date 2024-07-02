# HTTP Broker Configuration for AKRI

## Introduction

This guide provides detailed instructions on how to configure the http broker using AKRI. AKRI helps to expose IoT devices as resources in a Kubernetes cluster, and this configuration specifically focuses on integrating device sensors.

## Prerequisites

- A Kubernetes cluster
- Helm 3
- AKRI installed on your cluster

## Configuration File Overview

The `device-broker-config.yaml.template` file is a Kubernetes custom resource definition (CRD) that tells AKRI how to discover and use device sensors. This file includes several configuration options that can be customized according to your needs.

## Configuration Options

Below is an example of the template used to configure the http broker for the device sensor. You can change the values in the corresponding broker configuration for your specific environment.

```yaml
apiVersion: akri.sh/v0
kind: Configuration
metadata:
  name: akri-http-device
spec:
  discoveryHandler: 
    name: akri-http-discovery-handler # name of the http device discovery handler
    discoveryDetails: |+
      httpDevices:
      - id: http-device-1
        endpoint: "http://http-device-001:8080/device/input"     
      - id: http-device-2
        endpoint: "http://http-device-002:8080/device/input"             
  brokerSpec:
    brokerPodSpec:
      serviceAccountName: mqtt-client
      containers:
      - name: akri-http-broker # name of the http broker
        image: <container registry>/http-broker:latest # CHANGE TO YOUR CONTAINER REGISTRY
        env:
          - name: DATA_MQ_ENDPOINT
            value: "aio-mq-dmqtt-frontend"
          - name: MQ_PORT
            value: "8883"
          - name: MQ_DATA_TOPIC
            value: "http-device-data/input"
          - name: MQ_ERROR_TOPIC
            value: "http-device-data/input-error"           
          - name: SSL_CERT_FILE
            value: "/var/run/certs/ca.crt"
          - name: MQ_SAT_TOKEN
            value: "/var/run/secrets/tokens/mq-sat"
        resources:
          requests:
            "{{PLACEHOLDER}}" : "1"
            memory: 11Mi
            cpu: 10m
          limits:
            "{{PLACEHOLDER}}" : "1"
            memory: 24Mi
            cpu: 24m
        volumeMounts:
        - name: mq-sat
          mountPath: /var/run/secrets/tokens
        - name: trust-bundle
          mountPath: /var/run/certs
      volumes:
      - name: mq-sat
        projected:
          sources:
          - serviceAccountToken:
              path: mq-sat
              audience: aio-mq # Must match audience in BrokerAuthentication
              expirationSeconds: 86400
      - name: trust-bundle
        configMap:
          name: aio-ca-trust-bundle-test-only # Default root CA cert
      imagePullSecrets:
      - name: acr-auth
  instanceServiceSpec:
    type: ClusterIP
    ports:
    - name: akri-custom-instance-service
      port: 6052
      protocol: TCP
      targetPort: 6052
  configurationServiceSpec:
    type: ClusterIP
    ports:
    - name: akri-custom-configuration-service
      port: 6052
      protocol: TCP
  brokerProperties:
    json_schema: |
        {
          "type": "object",
          "properties": {
            "temperature": { "type": "number" },
            "humidity": { "type": "number" }
          },
          "required": ["temperature", "humidity"]
        }
  capacity: 1 # maximum number of nodes that can schedule workloads on a resource
```

- `apiVersion`: Specifies the API version used for the configuration file.
- `kind`: Specifies the kind of Kubernetes object, in this case, `Configuration`.
- `metadata`: Provides metadata about the configuration, such as its name.
- `spec`: Defines the specification for discovering and utilizing the device sensors.
  - `protocol`: Defines the protocol used by the device sensors.
  - `discoveryHandler`: Specifies the method used for discovering the sensors.
  - `properties`: Lists any additional properties required for sensor discovery and configuration.

## Configuration Options Detailed Explanation

### Environment Variables and Broker Properties

- **Environment Variables**: Set in the `brokerPodSpec` section under `containers -> env`. They configure the operation of the container application specified by the `image` field. For MQTT communication with Azure IoT Operations (AIO), the following environment variables are used:
    - `DATA_MQ_ENDPOINT`: Specifies the MQTT endpoint URL.
    - `MQ_PORT`: Specifies the port for the MQTT endpoint.
    - `MQ_DATA_TOPIC` and `MQ_ERROR_TOPIC`: Define the MQTT topics for sending data and errors, respectively.
    - `SSL_CERT_FILE`: Specifies the path to the SSL certificate for secure communication.
    - `MQ_SAT_TOKEN`: Specifies the path to the Service Account Token (SAT) for authentication.

### Volumes and Volume Mounts

- **Volumes**: Defined under `brokerPodSpec -> volumes`, they are used for mounting data into pods. In this configuration:
    - `mq-sat`: Mounts Service Account Tokens for authentication.
    - `trust-bundle`: Mounts SSL certificates for secure communication.
- **Volume Mounts**: Specified under `containers -> volumeMounts`, they define where the volumes are mounted inside the container. For example:
    - `mq-sat` is mounted at `/var/run/secrets/tokens`.
    - `trust-bundle` is mounted at `/var/run/certs`.

### JSON Schema

- **`json_schema`**: Defined under `brokerProperties`, it specifies the [JSON schema validation](generic-http-broker-design.md#json-schema-example) rules for validating the JSON response from the `httpDevice` endpoint. This ensures that the data received from devices matches the expected format.

### Resources

- **Resources**: Specified under `containers -> resources`, this section sets limits on the CPU and memory resources used by the pods in the cluster. It helps manage resource allocation and ensures that the broker does not consume more than the specified limits.

### `instanceServiceSpec` and `configurationServiceSpec`

- **`instanceServiceSpec`**: Defines the specifications for the service that exposes the instances of the discovered devices. It uses `ClusterIP` to make the service accessible within the cluster on a specified port.
- **`configurationServiceSpec`**: Similar to `instanceServiceSpec`, it defines the specifications for the service that exposes the configuration interface. It also uses `ClusterIP` and makes the service accessible within the cluster on a specified port.

### Capacity

- **`capacity`**: This parameter under the main `spec` section defines the maximum number of nodes that can schedule workloads on a resource. It limits the number of instances to prevent overallocation of resources. 

> **Tip:** When modifying the Configuration, do not remove the resource request and limit `{{PLACEHOLDER}}`. The Akri Controller inserts the request for the discovered device/Instance here.

## Additional Configuration Files

If there are additional configuration files in the repository, describe their purpose and usage here.

## Step-by-Step Configuration Guide

1. Ensure all prerequisites are met.
2. If providing a [JSON Schema](generic-http-broker-design.md#json-schema-example) for validation, please include it in the `/infra/akri/deploy/` folder.
3. Refer to the steps for [Applying the Configuration](managing-http-devices.md#applying-the-configuration)

## Troubleshooting
- If the sensors are not discovered, ensure that the discoveryHandler is correctly configured and that your sensors are compatible with the specified protocol.

## Further Resources
- [AKRI Custom Broker Documentation](https://docs.akri.sh/development/broker-development)
- [AIO MQTT Broker Documentation](https://learn.microsoft.com/en-us/azure/iot-operations/manage-mqtt-connectivity/overview-iot-mq)