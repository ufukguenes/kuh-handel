use kuh_handel_lib::{
    client::{self, Client},
    player::random_player::RandomPlayerActions,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::simple_player::SimplePlayer;
pub mod simple_player;

// Remove #[tokio::main] and manually set up the runtime
fn main() {
    let base_url = "://127.0.0.1:2000".to_string();

    let simple_bot = SimplePlayer::new("ufuk".to_string(), 0.1);

    let client: Client = Client {
        name: "ufuk".to_string(),
        token: "abcd".to_string(),
        bot: Box::new(simple_bot),
        base_url,
        last_ranking: Vec::new(),
    };

    /*
        // 1. Manually build a single-threaded runtime
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        // 2. Block on the LocalSet within the single-threaded runtime
        rt.block_on(async move {
            let local_set = tokio::task::LocalSet::new();

            // 3. Run all your tasks inside the LocalSet.
            local_set
                .run_until(async move {
                    // Note: start is now a synchronous function that spawns the local task.
                    let ufuk_handle = spawn_player("ufuk".to_string(), base_url.clone());
                    let leon_handle = spawn_player("leon".to_string(), base_url.clone());
                    let johannes_handle = spawn_player("johannes".to_string(), base_url.clone());
                    let viola_handle = spawn_player("viola".to_string(), base_url.clone());

                    // 4. Await the JoinHandles for the local tasks
                    let _ = tokio::join!(ufuk_handle, leon_handle, johannes_handle, viola_handle);
                })
                .await;
        });
    */
}

// Rename and restructure to be a synchronous function that calls spawn_local
pub fn spawn_player(id: String, base_url: String) -> tokio::task::JoinHandle<()> {
    let client: Arc<Mutex<Client>> = Arc::new(Mutex::new(Client {
        name: id.clone(),
        token: "abcd".to_string(),
        bot: Box::new(RandomPlayerActions::new(id, 3)),
        base_url: base_url.clone(),
        last_ranking: Vec::new(),
    }));

    // Use spawn_local directly, which works because we are inside a LocalSet
    tokio::task::spawn_local(async move {
        let _ = client.clone().lock().await.register().await;
        client.clone().lock().await.play_one_round().await;
    })
}
