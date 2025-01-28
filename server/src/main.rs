use tokio::net::UdpSocket;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("== Start Yatch Server ==");
    let sock = UdpSocket::bind("0.0.0.0:8080").await?;
    println!("LOG: Listening to socket 0.0.0.0:8080");
    let mut buf = [0; 1024];
    loop {
        let (len, addr) = sock.recv_from(&mut buf).await?;
        println!("LOG: {:?} bytes received from {:?}", len, addr);

        let len = sock.send_to(&buf[..len], addr).await?;
        println!("LOG: {:?} bytes sent", len);
    }
}
