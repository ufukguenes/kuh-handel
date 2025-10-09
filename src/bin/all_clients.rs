use kuh_handel::{
    client_side::client::Client, model::player::player_actions::random_actions::RandomPlayerActions,
};

#[tokio::main]
async fn main() {
    let ufuk_string = "ufuk".to_string();
    let ufuk_client = Client {
        name: ufuk_string.clone(),
        bot: RandomPlayerActions::new(ufuk_string, 3),
        print_indent_size: 0,
    };

    let leon_string = "leon".to_string();
    let leon_client = Client {
        name: leon_string.clone(),
        bot: RandomPlayerActions::new(leon_string, 42),
        print_indent_size: 1,
    };

    let ufuk_handel = tokio::spawn(ufuk_client.start());
    let leon_handel = tokio::spawn(leon_client.start());

    while !ufuk_handel.is_finished() || !leon_handel.is_finished() {}
}
