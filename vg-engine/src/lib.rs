use winit::event::Event;

#[cfg(target_os = "android")]
pub mod platform {
    mod android;
    pub use android::*;
}

#[cfg(not(target_os = "android"))]
pub mod platform {
    mod desktop;
    pub use desktop::*;
}

pub enum UserEvent {}

pub struct Engine {}

impl Engine {
    pub fn new() -> Engine {
        Engine {}
    }

    pub fn event(&mut self, _event: Event<UserEvent>) {

    }
}

/*
async fn run_host(mut instance: impl Instance, mut socket: Socket) -> Result<()> {
    let mut host = HostData::new();
    let mut instant = Instant::now();

    loop {
        tokio::task::yield_now().await;

        host.poll(&mut socket)?;

        // It is time for a server tick
        let elapsed = instant.elapsed();
        if elapsed >= Duration::from_millis(1000) {
            // Execute one tick
            loop {
                match instance.step() {
                    WaitReason::Startup => continue,
                    WaitReason::Present => break,
                }
            }

            // Announce new tick
            let data = instance.get_data();
            host.tick(&mut socket, &data, elapsed)?;

            instant += elapsed;
        }
    }
}

async fn run_client(mut instance: impl Instance, mut socket: Socket) -> Result<()> {
    let mut client = ClientData::new();

    loop {
        tokio::task::yield_now().await;

        let Some(confirm) = client.poll(&mut socket)? else { continue };

        // Deserialize state if server pushed
        if let Some(data) = confirm.state()? {
            instance.set_data(&data);
        }

        // Execute one tick
        loop {
            match instance.step() {
                WaitReason::Startup => continue,
                WaitReason::Present => break,
            }
        }

        let data = instance.get_data();
        // Desync
        if confirm.diverged(&data) {
            client.desync(&mut socket)?;
        }
    }
}

*/
