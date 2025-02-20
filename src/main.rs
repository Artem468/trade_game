use trade_game::run;

#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => {
            println!("Processed successfully!");
        }
        Err(err) => {
            println!("Something went wrong! ({err})");
        }
    }
}
