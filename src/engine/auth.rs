use scram::ScramClient;
use tokio::io::{AsyncRead, AsyncWrite};

use super::AuthConfig;
use super::Rustbase;

pub async fn auth<IO>(auth_config: AuthConfig, client: &mut IO)
where
    IO: AsyncRead + AsyncWrite + Unpin,
{
    let scram = ScramClient::new(&auth_config.username, &auth_config.password, None);
    let (scram, client_first) = scram.client_first();

    let server_first = Rustbase::send_and_receive(client, client_first.as_bytes().to_vec()).await;
    let server_first = String::from_utf8(server_first).unwrap();

    let scram = scram
        .handle_server_first(&server_first)
        .expect("Invalid server first message, maybe server is not using scram? or maybe the username is wrong?");
    let (scram, client_final) = scram.client_final();

    let server_final = Rustbase::send_and_receive(client, client_final.as_bytes().to_vec()).await;
    let server_final = String::from_utf8(server_final).unwrap();

    scram.handle_server_final(&server_final).unwrap();
}
