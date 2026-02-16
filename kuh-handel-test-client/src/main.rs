use kuh_handel_lib::{client::Client, player::simple_player::SimplePlayer};
use std::sync::Arc;
use tokio::sync::Mutex;

fn main() {
    let base_url = "://127.0.0.1:2000".to_string();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async move {
        let local_set = tokio::task::LocalSet::new();

        local_set
            .run_until(async move {
                let ufuk_handle = spawn_player("ufuk".to_string(), base_url.clone(), 0.1);
                let leon_handle = spawn_player("leon".to_string(), base_url.clone(), 0.2);
                let johannes_handle = spawn_player("johannes".to_string(), base_url.clone(), 0.3);
                let viola_handle = spawn_player("viola".to_string(), base_url.clone(), 0.4);

                let _ = tokio::join!(ufuk_handle, leon_handle, johannes_handle, viola_handle);
            })
            .await;
    });
}

pub fn spawn_player(id: String, base_url: String, risk: f32) -> tokio::task::JoinHandle<()> {
    let simple_bot = SimplePlayer::new(id.clone(), risk);
    let client: Arc<Mutex<Client>> = Arc::new(Mutex::new(Client::new(
        id.clone(),
        "abcd".to_string(),
        Box::new(simple_bot),
        base_url.clone(),
    )));

    tokio::task::spawn_local(async move {
        let _ = client.clone().lock().await.register().await;

        for _ in 0..10 {
            client
                .clone()
                .lock()
                .await
                .play_one_round("pvp_games".to_string())
                .await;
        }
    })
}
