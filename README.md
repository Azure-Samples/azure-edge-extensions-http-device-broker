# Custom HTTP Device Discovery Handler in AKRI

## Introduction

This guide outlines the steps to create a custom HTTP device discovery handler in AKRI. AKRI simplifies the process of exposing IoT devices as resources in a Kubernetes cluster. This custom handler will enable the discovery of HTTP-based device sensors.

## Prerequisites

- A Kubernetes cluster
- Helm 3
- AKRI installed on your cluster

## Overview of Creating a Custom Discovery Handler

Creating a custom discovery handler involves several key steps:

1. **Designing the Discovery Logic**: Determine how your handler will discover devices. This involves defining the criteria and method for identifying available HTTP-based devices in your network.

2. **Implementing the Discovery Handler**: Write the code for your discovery handler, adhering to AKRI's discovery handler interface. This code will be responsible for scanning the network based on your defined logic and reporting discovered devices back to AKRI.

3. **Containerizing the Discovery Handler**: Package your discovery handler as a Docker container. This involves creating a Dockerfile that specifies the base image, dependencies, and the command to run your handler.

4. **Deploying the Discovery Handler**: Deploy your containerized discovery handler to your Kubernetes cluster. This step involves creating Kubernetes manifests for your handler and applying them to your cluster.

5. **Configuring AKRI to Use Your Discovery Handler**: Update AKRI's configuration to use your custom discovery handler for device discovery. This involves modifying the AKRI Configuration CRD to specify your handler.

## Discovery Handler Discovery Details Settings

- The HTTP Discovery Handler identifies the characteristics of the HTTP-based devices you want to discover (e.g., specific endpoints, response patterns). The Discovery Handler will be responsible for scanning the network based on your defined logic and reporting discovered devices back to AKRI. It requires a set of DiscoveryURLs to determine if an endpoint is listening and valid. The device endpoint url and a unique device id represents the http device.

#### Discovery Object

|Helm Key|Value|Default|Description
|--------|-----|-------|-----------
|http.configuration.discoveryDetails.devices|array of device objects|[{_device object_}]|An array of device objects used in defining the devices

|Helm Key|Value|Default|Description
|--------|-----|-------|-----------
|http.configuration.discoveryDetails.devices[].discoveryUrls|array of DiscoveryURLs|["http://localhost:4840/"]|Endpoints that are the status URLs to check to see if the device is up
|http.configuration.discoveryDetails.devices[].deviceIds|array of Device Identifiers|["http-device-001"]|A unique identifier for the http device


## Adding a Device

To add an http device, you need to add a new entry under `httpDevices` in the AKRI configuration. Each entry should have the following fields:

- `id`: A unique identifier for the device.
- `endpoint`: The URL where the device can be accessed.

Here's an example of how to add a device:

```yaml
httpDevices:
- id: new-device
  endpoint: "http://new-device:8080/new-device/input"

```

## Removing a Device

To remove a device, simply delete its entry from `httpDevices` in the AKRI configuration.

## Configuring AKRI to Use the Discovery Handler

- Modify the [AKRI Broker Configuration](broker-config.md) to use your custom discovery handler. This guide provides details on how to specify the name of your device handler and any necessary parameters. In addtion, it allows you to apply the updated AKRI Configuration to your cluster.

## Troubleshooting

- Ensure your discovery handler is correctly implemented and adheres to AKRI's interface.
- Verify that your Docker image is accessible to your Kubernetes cluster.
- Check the logs of your discovery handler pod for any errors during the discovery process.

## Further Resources

- [AKRI Documentation](https://docs.akri.sh/)
- [Design Patterns: Publisher-Subscriber pattern](https://learn.microsoft.com/en-us/azure/architecture/patterns/publisher-subscriber)
- [What is MQTT and How Does it Work](https://www.techtarget.com/iotagenda/definition/MQTT-MQ-Telemetry-Transport)
- [Configure TLS with manual certificate management to secure MQTT communication](https://learn.microsoft.com/en-us/azure/iot-operations/manage-mqtt-connectivity/howto-configure-tls-manual)
- [Detect assets with Azure IoT Akri](https://learn.microsoft.com/en-us/azure/iot-operations/manage-devices-assets/overview-akri)
