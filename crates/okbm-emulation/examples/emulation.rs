use okbm_emulation::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut emulation = Emulation::new()?;
    emulation.create(0).await;

    loop {
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

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        emulation
            .consume(
                Event::Pointer(PointerEvent::Motion {
                    time: 0,
                    dx: -100.0,
                    dy: 100.0,
                }),
                0,
            )
            .await?;

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
