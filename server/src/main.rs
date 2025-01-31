use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("== Start Yatch Server ==");
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("LOG: Listening to socket 0.0.0.0:8080");

    loop {
        let (mut socket, _) = listener.accept().await?;
        println!("LOG: New socket connection!");

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };
                println!("LOG: Recv n={}", n);

                // Write the data back
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
                println!("LOG: Write back n={}", n);
            }
        });
    }
}
