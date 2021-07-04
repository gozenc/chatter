use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;
fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}
fn main(){
    let server = TcpListener::bind(LOCAL)
        .expect("Listener failed to bind.");
    server.set_nonblocking(true)
        .expect("Failed to initialize non-blocking.");

    let mut clients: Vec<TcpStream> = vec![];
    let (sender, receiver) = mpsc::channel::<String>();

    loop {
        // If server accepts, then returns socket and its address:
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected.", addr);

            let sender = sender.clone();
            clients.push(socket.try_clone().expect("Failed to clone client."));

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];
                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg)
                        .expect("Invalid UTF8 message.");
                        println!("{}: {:?}", addr, msg);
                        sender.send(msg).expect("Failed to send message to receiver.")
                    },
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Closing connection with: {}", addr);
                        break;
                    }
                }
                sleep();
            });
        }

        // When server receives a message, it tries to receive within channel.
        if let Ok(msg) = receiver.try_recv() {
            clients = clients.into_iter().filter_map(|mut client| {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).map(|_| client).ok()
            }).collect::<Vec<_>>();
        }

        sleep();

    }
}
