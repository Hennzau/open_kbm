use okbm::*;

use rkyv::{Archive, Deserialize, Serialize, rancor::Error};

#[derive(Debug, PartialEq, Clone, Copy, Archive, Serialize, Deserialize)]
pub struct ZenohEvent {
    handle: u32,
    event: CaptureEvent,
}

#[tokio::main]
async fn main() -> Result<()> {
    let id = "192.168.1.49".to_string();
    let position = Position::Right;
    let neighbour = "192.168.1.34".to_string();

    zenoh::try_init_log_from_env();

    let mut config = zenoh::Config::default();
    config
        .insert_json5("connect/endpoints", r#"["udp/192.168.1.34:4242"]"#)
        .map_err(Report::msg)?;

    config
        .insert_json5("listen/endpoints", r#"["udp/192.168.1.49:4242"]"#)
        .map_err(Report::msg)?;

    let session = zenoh::open(config).await.map_err(Report::msg)?;

    let subscriber = session
        .declare_subscriber(format!("okbm/{}", id))
        .await
        .map_err(Report::msg)?;

    let publisher = session
        .declare_publisher(format!("okbm/{}", neighbour))
        .await
        .map_err(Report::msg)?;

    let mut capture = Capture::new().await?;
    capture.create(0, position).await?;

    let mut emulation = Emulation::new()?;
    emulation.create(0).await;

    loop {
        tokio::select! {
            Some(Ok(event)) = capture.next() => {
                if let CaptureEvent::Input(Event::Keyboard(KeyboardEvent::Key { key: 1, .. })) = event.1 {
                    capture.release().await?;

                    continue;
                }

                let event = ZenohEvent { handle: event.0, event: event.1 };

                let bytes = rkyv::to_bytes::<Error>(&event)?;

                publisher.put(bytes.as_slice()).await.map_err(Report::msg)?;
            }

            Ok(message) = subscriber.recv_async() => {
                let bytes = message.payload().to_bytes();

                let archived = rkyv::access::<ArchivedZenohEvent, Error>(&bytes[..])?;
                let message = rkyv::deserialize::<ZenohEvent, Error>(archived)?;

                let handle = message.handle;
                let event = message.event;

                match event {
                    CaptureEvent::Begin => {
                        capture.release().await?;
                    }
                    CaptureEvent::Input(event) => {
                        emulation.consume(event, handle).await?;
                    }
                }
            }
        }
    }
}
