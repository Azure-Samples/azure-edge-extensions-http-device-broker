use reqwest::get;
use std::collections::HashMap;

const DEVICE_ENDPOINT: &str = "AKRI_HTTP_DEVICE_ENDPOINT";
const DEVICE_ID: &str = "DEVICE_ID";

use akri_discovery_utils::discovery::{
    discovery_handler::{deserialize_discovery_details},
    v0::{discovery_handler_server::DiscoveryHandler, Device, DiscoverRequest, DiscoverResponse},
    DiscoverStream,
};
use async_trait::async_trait;
use tokio::sync::mpsc;
use tonic::{Response, Status};
use tokio::time::sleep;
use std::time::Duration;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct HttpDevice {
    id: String,
    endpoint: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct DiscoveryDetails {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    http_devices: Vec<HttpDevice>,
}

pub struct DiscoveryHandlerImpl {
    register_sender: tokio::sync::mpsc::Sender<()>,
}

impl DiscoveryHandlerImpl {
    pub fn new(register_sender: tokio::sync::mpsc::Sender<()>) -> Self {
        DiscoveryHandlerImpl { register_sender }
    }
}

#[async_trait]
impl DiscoveryHandler for DiscoveryHandlerImpl {
    type DiscoverStream = DiscoverStream;
    async fn discover(
        &self,
        request: tonic::Request<DiscoverRequest>,
    ) -> Result<Response<Self::DiscoverStream>, Status> {

        let discover_request = request.get_ref();
        let discovery_handler_config: DiscoveryDetails =
            deserialize_discovery_details(&discover_request.discovery_details)
                .map_err(|e| tonic::Status::new(tonic::Code::InvalidArgument, format!("{}", e)))?;

        let device_list = discovery_handler_config.http_devices.clone();
        println!("Devices: {:?}", device_list);

        // For each device, print the device id, endpoint, and topic
        for device in &device_list {
            println!("Device id: {}", device.id);
            println!("Device endpoint: {}", device.endpoint);
        }

        // Create a channel for sending and receiving device updates
        let (stream_sender, stream_receiver) = mpsc::channel(4);
        let register_sender = self.register_sender.clone();
        tokio::spawn(async move {
            loop {    

                // Create a new list of devices based on if the device is online or offline
                let mut online_devices = Vec::new();
                for device in &device_list {
                    let url = device.endpoint.clone();
                    let response = get(&url).await;
                    if let Ok(response) = response {
                        if response.status().is_success() {
                            online_devices.push(device.clone());
                        }
                    }
                }

                // Build Device for each device in the device list
                let devices = online_devices
                    .iter()
                    .map(|device| {
                        let mut properties = HashMap::new();
                        properties.insert(DEVICE_ENDPOINT.to_string(), device.endpoint.to_string());
                        properties.insert(DEVICE_ID.to_string(), device.id.to_string());
                        Device {
                            id: device.id.to_string(),
                            properties,
                            mounts: Vec::default(),
                            device_specs: Vec::default(),
                        }
                    })
                    .collect::<Vec<Device>>();
                    
                // Print list of devices
                println!("Devices: {:?}", devices);

                if let Err(_) = stream_sender.send(Ok(DiscoverResponse { devices })).await {
                    // Agent dropped its end of the stream. Stop discovering and signal to try to re-register.
                    register_sender.send(()).await.unwrap();
                    break;
                }

                // wait 10 sec
                sleep(Duration::from_secs(10)).await;
            }
        });
        // Send the agent one end of the channel to receive device updates
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
            stream_receiver,
        )))
    }
}
