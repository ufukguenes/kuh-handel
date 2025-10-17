use kuh_handel_lib::client::Client;
use kuh_handel_lib::player::random_player::RandomPlayerActions;

#[tokio::main]
async fn main() {
    let ufuk_string = "ufuk".to_string();
    let ufuk_client = Client {
        name: ufuk_string.clone(),
        token: "abcd".to_string(),
        bot: RandomPlayerActions::new(ufuk_string, 3),
    };

    let leon_string = "leon".to_string();
    let leon_client = Client {
        name: leon_string.clone(),
        token: "efgh".to_string(),
        bot: RandomPlayerActions::new(leon_string, 42),
    };

    let johannes_string = "johannes".to_string();
    let johannes_client = Client {
        name: johannes_string.clone(),
        token: "ijkl".to_string(),
        bot: RandomPlayerActions::new(johannes_string, 42),
    };

    let viola_string = "viola".to_string();
    let viola_client = Client {
        name: viola_string.clone(),
        token: "mnop".to_string(),
        bot: RandomPlayerActions::new(viola_string, 42),
    };

    let _ = ufuk_client.register().await;
    let _ = leon_client.register().await;
    let _ = johannes_client.register().await;
    let _ = viola_client.register().await;

    let ufuk_handel = tokio::spawn(ufuk_client.start());
    let leon_handel = tokio::spawn(leon_client.start());
    let johannes_handel = tokio::spawn(johannes_client.start());
    let viola_handel = tokio::spawn(viola_client.start());

    let _ = ufuk_handel.await;
    let _ = leon_handel.await;
    let _ = johannes_handel.await;
    let _ = viola_handel.await;
}
