use std::sync::Arc;

use kuh_handel_lib::client::Client;
use kuh_handel_lib::player::random_player::RandomPlayerActions;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let base_url = "://127.0.0.1:2000".to_string();

    let ufuk_string = "ufuk".to_string();
    let ufuk_client = Arc::new(Mutex::new(Client {
        name: ufuk_string.clone(),
        token: "abcd".to_string(),
        bot: RandomPlayerActions::new(ufuk_string, 3),
        base_url: base_url.clone(),
    }));

    let leon_string = "leon".to_string();
    let leon_client = Arc::new(Mutex::new(Client {
        name: leon_string.clone(),
        token: "efgh".to_string(),
        bot: RandomPlayerActions::new(leon_string, 42),
        base_url: base_url.clone(),
    }));

    let johannes_string = "johannes".to_string();
    let johannes_client = Arc::new(Mutex::new(Client {
        name: johannes_string.clone(),
        token: "ijkl".to_string(),
        bot: RandomPlayerActions::new(johannes_string, 42),
        base_url: base_url.clone(),
    }));

    let viola_string = "viola".to_string();
    let viola_client = Arc::new(Mutex::new(Client {
        name: viola_string.clone(),
        token: "mnop".to_string(),
        bot: RandomPlayerActions::new(viola_string, 42),
        base_url: base_url.clone(),
    }));

    let _ = ufuk_client.clone().lock().await.register().await;
    let _ = leon_client.clone().lock().await.register().await;
    let _ = johannes_client.clone().lock().await.register().await;
    let _ = viola_client.clone().lock().await.register().await;

    let start_client = |client: Arc<Mutex<Client>>| async move {
        client.clone().lock().await.play_one_round().await
    };

    let ufuk_handel = tokio::spawn(start_client(ufuk_client));
    let leon_handel = tokio::spawn(start_client(leon_client));
    let johannes_handel = tokio::spawn(start_client(johannes_client));
    let viola_handel = tokio::spawn(start_client(viola_client));

    let _ = ufuk_handel.await;
    let _ = leon_handel.await;
    let _ = johannes_handel.await;
    let _ = viola_handel.await;
}
