use std::io::{Read, Write};
use std::net::TcpStream;
use crypto_box::SalsaBox;
use websocket::sync::stream::Splittable;
use websocket::sync::Client;
use websocket::{Message, OwnedMessage};
use crypto_box::{
    aead::{Aead, AeadCore, OsRng}, PublicKey, SecretKey
};

pub fn handle(stream: TcpStream, mut client: Client<TcpStream>) {

    let secret_key = SecretKey::generate(&mut OsRng);
    let public_key_bytes = secret_key.public_key().as_bytes().clone();
    log::info!("Starting key exchange");

    // send our public key
    client.send_message(&OwnedMessage::Binary(public_key_bytes.to_vec())).unwrap();
    log::debug!("Sent our pub key: {:?}", public_key_bytes);
    // recv their public key
    let other_public_key = match client.recv_message() {
        Ok(OwnedMessage::Binary(data)) => PublicKey::from(<[u8; 32]>::try_from(data).unwrap()),
        other => panic!("key xchange: {:?}", other),
    };
    log::debug!("Got their pub key: {:?}", other_public_key);

    let enc_box = SalsaBox::new(&other_public_key, &secret_key);
    let dec_box = SalsaBox::new(&other_public_key, &secret_key);

    let (mut c_rx, mut c_tx) = client.split().unwrap();
    let (mut s_rx, mut s_tx) = stream.split().unwrap();

    // stream -> client
    std::thread::spawn(move || loop {
        let mut buf = vec![0; 1 << 20];

        match s_rx.read(&mut buf) {
            Ok(n_bytes) => {
                log::debug!("sending bytes: {:?}", &buf[..n_bytes]);
                let nonce = SalsaBox::generate_nonce(&mut OsRng);
                let mut msg = Vec::with_capacity(128);
                msg.extend_from_slice(&(nonce.len() as u32).to_le_bytes());
                msg.extend_from_slice(nonce.as_slice());
                let encrypted = enc_box.encrypt(&nonce, &buf[..n_bytes]).unwrap();
                msg.extend_from_slice(&encrypted);
                log::debug!("sending encrypted bytes: {:?}", msg);
                c_tx.send_message(&Message::binary(msg)).unwrap();
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => {
                panic!("got {:?}", e);
            }
        }
    });

    // client -> stream
    std::thread::spawn(move || 'handle_loop: loop {
        match c_rx.recv_message() {
            Ok(OwnedMessage::Binary(data)) => {
                log::debug!("recv encrypted bytes: {:?}", data);
                let len = u32::from_le_bytes(data[..4].try_into().unwrap()) as usize;
                let nonce = &data[4..4 + len];
                let encrypted = &data[4 + len..];
                let decrypted = dec_box.decrypt(nonce.into(), encrypted).unwrap();
                log::debug!("decrypted bytes: {:?}", decrypted);
                s_tx.write_all(&decrypted).unwrap();
            }
            Ok(OwnedMessage::Close(_)) => {
                break 'handle_loop;
            }
            Ok(data) => {
                panic!("got {:?}", data);
            }
            Err(websocket::WebSocketError::NoDataAvailable) => {}
            Err(e) => {
                panic!("err: {:?}", e);
            }
        }
    });
}