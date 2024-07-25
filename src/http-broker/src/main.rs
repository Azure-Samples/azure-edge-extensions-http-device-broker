use reqwest::get;
use std::env;
use serde_json::Value;
use tokio::{time, time::Duration, task};
use rumqttc::{MqttOptions, AsyncClient as MqttClient, QoS, Transport};
use tokio_rustls::rustls::ClientConfig;
mod error;
use crate::error::{ErrorAlert, validate_json};
use crate::error::parse_json_schema;

const AKRI_HTTP_DEVICE_ENDPOINT_VAR: &str = "AKRI_HTTP_DEVICE_ENDPOINT";
const DATA_MQ_ENDPOINT_VAR: &str = "DATA_MQ_ENDPOINT";
const MQ_PORT_VAR: &str = "MQ_PORT";
const MQ_DATA_TOPIC_VAR: &str = "MQ_DATA_TOPIC";
const MQ_ERROR_TOPIC_VAR: &str = "MQ_ERROR_TOPIC";
const DEVICE_ID_VAR: &str = "DEVICE_ID";
const POLLING_INTERVAL_VAR: &str = "POLLING_INTERVAL";

const ERROR_PARSING_JSON: &str = "300";
const ERROR_GETTING_DATA: &str = "200";

async fn read_sensor(device_url: &str, publish_endpoint: &str, port: u16, mq_error_topic: &str, mq_data_topic: &str, json_schema: Value, device_id: &str) {
  println!("Reading sensor data");
  let mut mqtt_options = MqttOptions::new("http-broker-client", publish_endpoint, port);
  mqtt_options.set_keep_alive(std::time::Duration::from_secs(5));

  let username = "$sat";
  let password_file = std::env::var("MQ_SAT_TOKEN").expect("MQTT_PASSWORD environment variable not set");
  let password = std::fs::read_to_string(password_file).expect("Failed to read password file");
  mqtt_options.set_credentials(username, password);

  // Load  certificate from the file system
  // Use rustls-native-certs to load root certificates from the operating system.
  let mut root_cert_store = tokio_rustls::rustls::RootCertStore::empty();
  root_cert_store.add_parsable_certificates(
      rustls_native_certs::load_native_certs().expect("could not load platform certs"),
  );

  let client_config = ClientConfig::builder()
      .with_root_certificates(root_cert_store)
      .with_no_client_auth();
  mqtt_options.set_transport(Transport::tls_with_config(client_config.into()));

  let (mqtt_client, mut eventloop) = MqttClient::new(mqtt_options.clone(), 10);   
  println!("get data from: {}", device_url);
  match get(device_url).await {
    Ok(resp) => {
      let body = resp.text().await.unwrap().trim_end_matches('\n').to_string();
      match serde_json::from_str::<Value>(&body) { 
        Ok(json) => {
          println!("[main:read_sensor] JSON response: {:?}", json);
          match validate_json(json_schema, json, device_id) {
            Ok(_) => {
                mqtt_publish_message(mq_data_topic, body.clone(), mqtt_client.clone()).await;
            },
            Err(alert) => {
              println!("Generating json schema validation error alert: {}", alert);
              mqtt_publish_message(mq_error_topic, alert, mqtt_client.clone()).await;
            },
          }        
        },
        Err(err) => {
          let error_message = format!("Failed to parse JSON from sensor: {:?}", err);
          println!("{}", error_message);
          let error_alert = ErrorAlert::generate_alert(device_id.to_string(), ERROR_PARSING_JSON.to_string(), error_message.to_string());
          println!("Generating Data Not Available error alert: {}", error_alert);
          mqtt_publish_message(mq_error_topic, error_alert, mqtt_client.clone()).await;
        },
      }
    },
    Err(_) => {
      let error_message = "Failed to get data from sensor";
      println!("{}", error_message);
      let error_alert = ErrorAlert::generate_alert(device_id.to_string(), ERROR_GETTING_DATA.to_string(), error_message.to_string());
      println!("Generating Device Not Responding error alert: {}", error_alert);
      mqtt_publish_message(mq_error_topic, error_alert, mqtt_client.clone()).await;
    }
  }

  let mut counter = 0;
  loop {
      // Waits for and retrieves the next event in the event loop.
      let event = eventloop.poll().await;
      // Performs pattern matching on the retrieved event to determine its type
      match &event {
          Ok(v) => {
              println!("Event = {:?}", v);
              counter += 1;
              if counter % 2 == 0 {
                  // Message is published here.
                  break;
              }
          }
          Err(e) => {
              println!("Error sending message = {:?}", e);
              break;
          }
      }
  }
}

async fn mqtt_publish_message(mq_topic: &str, message: String, client: MqttClient) {  
  let mq_topic = mq_topic.to_owned();
  task::spawn(async move {
    let mq_topic_clone = mq_topic.clone();
    task::spawn(async move {
      match client.publish(mq_topic_clone, QoS::AtLeastOnce, false, message.into_bytes()).await {
        Ok(_) => println!("Message successfully sent to topic: {}", mq_topic),
        Err(e) => println!("Failed to send message to topic: {}. Error: {}", mq_topic, e),
      }  
    });    
  });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting HTTP Broker, pulling configuration from environment variables");

    // This environment variable come with an id as suffix, e.g. AKRI_HTTP_DEVICE_ENDPOINT_424BA8
    let device_url = env::vars()
      .filter(|(key, _value)| key.starts_with(AKRI_HTTP_DEVICE_ENDPOINT_VAR))
      .map(|(_key, value)| value)
      .next()
      .expect("No AKRI_HTTP_DEVICE_ENDPOINT environment variable found");
      
    // This environment variable come with an id as suffix, e.g. DEVICE_ID_424BA8
    let device_id = env::vars()
      .filter(|(key, _value)| key.starts_with(DEVICE_ID_VAR))
      .map(|(_key, value)| value)
      .next()
      .expect("No DEVICE_ID environment variable found");

    let publish_endpoint = env::var(DATA_MQ_ENDPOINT_VAR)
      .unwrap()
      .parse::<String>()
      .expect("No DATA_MQ_ENDPOINT environment variable found");

    let publish_endpoint_port = env::var(MQ_PORT_VAR)
      .unwrap()
      .parse::<u16>()
      .expect("No MQ_PORT environment variable found");

    let mq_data_topic = env::var(MQ_DATA_TOPIC_VAR)
      .unwrap()
      .parse::<String>()
      .expect("No MQ_DATA_TOPIC environment variable found");

    let mq_error_topic = env::var(MQ_ERROR_TOPIC_VAR)
      .unwrap()
      .parse::<String>()
      .expect("No MQ_ERROR_TOPIC environment variable found");
      
    let json_schema = env::var("json_schema")
      .unwrap()
      .parse::<String>()
      .expect("No json_schema environment variable found");
    
    let polling_interval = env::var(POLLING_INTERVAL_VAR)
      .unwrap_or_else(|_| "10".to_string())
      .parse::<u64>()
      .expect("POLLING_INTERVAL must be a valid u64 number");

    println!("Starting HTTP Broker with the following configuration:");

    println!("Device URL: {}", device_url);
    println!("Device ID: {}", device_id);
    println!("Publish Endpoint: {}", publish_endpoint);
    println!("Publish Endpoint Port: {}", publish_endpoint_port);
    println!("MQ Data Topic: {}", mq_data_topic);
    println!("MQ Error Topic: {}", mq_error_topic);
    println!("Polling Interval: {}", polling_interval);

    let mut tasks = Vec::new();
    let device_url_clone = device_url.clone();
    let publish_endpoint_clone = publish_endpoint.clone();
    let mq_data_topic_clone = mq_data_topic.clone(); 
    let device_id_clone = device_id.clone();   
    let mq_error_topic_clone = mq_error_topic.clone(); 
    match parse_json_schema(&json_schema) {
      Ok(schema) => {
          println!("Successfully parsed JSON schema");    
        tasks.push(tokio::spawn(async move {
          println!("Starting sensor reading loop");
          loop {        
            read_sensor(&device_url_clone[..], &publish_endpoint_clone[..], publish_endpoint_port, &mq_error_topic_clone[..], &mq_data_topic_clone[..], schema.clone(), &device_id_clone[..]).await;
            println!("Sleeping for {} seconds", polling_interval);
            time::sleep(Duration::from_secs(polling_interval)).await;
          }
        }));
      },
      Err(e) => {
          println!("Failed to parse JSON schema: {}", e);
          return Err(e.into());
      }
    }        
    futures::future::join_all(tasks).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_read_sensor() {
        let _m = mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"temperature": 25.5, "humidity": 50.0}"#)
            .create();

        let device_url = server_url();
        let publish_endpoint = "aio-mq-dmqtt-frontend";
        let publish_endpoint_port = 8883;
        let mq_data_topic = "cvx/business_unit/facility/gateway_id/device_id/weather-station/input";
        let mq_error_topic = "cvx/business_unit/facility/gateway_id/device_id/weather-station/input-error";
        let json_schema = json!({
          "type": "object",
          "properties": {
            "temperature": { "type": "number" },
            "humidity": { "type": "number" }
          },
          "required": ["temperature", "humidity"]
        });
        let device_id = "weater-station";

        let mut tasks = Vec::new();

        tasks.push(tokio::spawn(async move {
          read_sensor(&device_url[..], &publish_endpoint[..], publish_endpoint_port, &mq_error_topic[..], &mq_data_topic[..], json_schema.clone(), &device_id[..]).await;
        }));  
  }
}
