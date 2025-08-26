use emulation::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut emulation = Emulation::new()?;
    emulation.create(0).await;

    emulation
        .consume(
            Event::Pointer(PointerEvent::Motion {
                time: 0,
                dx: 100.0,
                dy: -100.0,
            }),
            0,
        )
        .await?;

    Ok(())
}
