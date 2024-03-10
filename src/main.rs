use alesia_client::AlesiaClient;
pub mod types;

#[tokio::main]
async fn main() {
    let mut client = AlesiaClient::new_from_url("127.0.0.1:8080").await;

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "exit" {
            break;
        }

        let response = client.send_query(input, &[&1i32]).await.unwrap();
        println!("{:?}", response);
    }
}

