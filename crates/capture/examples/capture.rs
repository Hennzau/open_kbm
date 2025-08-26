use capture::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut capture = Capture::new().await?;
    capture.create(0, Position::Top).await?;

    while let Some(event) = capture.next().await {
        let event = event?;

        println!("{:?}", event);

        if let CaptureEvent::Input(Event::Keyboard(KeyboardEvent::Key { key: 1, .. })) = event.1 {
            capture.release().await?;

            break;
        }
    }

    Ok(())
}
