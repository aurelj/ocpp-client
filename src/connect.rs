use tokio::net::{TcpStream};
use tokio_tungstenite::{client_async_tls, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::{Request};
use tokio_tungstenite::tungstenite::http::header::{AUTHORIZATION, SEC_WEBSOCKET_PROTOCOL};
use url::Url;
use crate::client::Client;

#[cfg(feature = "ocpp_1_6")]
use crate::ocpp_1_6::OCPP1_6Client;

#[cfg(feature = "ocpp_2_0_1")]
use crate::ocpp_2_0_1::OCPP2_0_1Client;

/// Connect to an OCPP server using the best OCPP version available.
pub async fn connect(address: &str) -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
    let (stream, protocol) = setup_socket(address, "ocpp1.6, ocpp2.0.1").await?;

    match protocol.as_str() {
        #[cfg(feature = "ocpp_1_6")]
        "ocpp1.6" => {
            Ok(Client::OCPP1_6(OCPP1_6Client::new(stream)))
        }
        #[cfg(feature = "ocpp_2_0_1")]
        "ocpp2.0.1" => {
            Err("Not supported".into())
        }
        _ => {
            Err("Connected to unknown OCPP server".into())
        }
    }
}

/// Connect to an OCPP 1.6 server.
#[cfg(feature = "ocpp_1_6")]
pub async fn connect_1_6(address: &str) -> Result<OCPP1_6Client, Box<dyn std::error::Error + Send + Sync>> {
    let (stream, _) = setup_socket(address, "ocpp1.6").await?;
    Ok(OCPP1_6Client::new(stream))
}

/// Connect to an OCPP 2.0.1 server.
pub async fn connect_2_0_1(address: &str) -> Result<OCPP2_0_1Client, Box<dyn std::error::Error + Send + Sync>> {
    let (stream, _) = setup_socket(address, "ocpp2.0.1").await?;
    Ok(OCPP2_0_1Client::new(stream))
}

async fn setup_socket(address: &str, protocols: &str) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, String), Box<dyn std::error::Error + Send + Sync>>{
    let address = Url::parse(address)?;

    let socket_addrs = address.socket_addrs(|| None)?;
    let stream = TcpStream::connect(&*socket_addrs).await?;

    let mut request: Request<()> = address.to_string().into_client_request()?;
    request.headers_mut().insert(SEC_WEBSOCKET_PROTOCOL, protocols.parse()?);
    request.headers_mut().insert(AUTHORIZATION, "ocpp1.6".parse()?);

    let (stream, response) = client_async_tls(request, stream).await?;

    let protocol = response.headers().get(SEC_WEBSOCKET_PROTOCOL).ok_or("No OCPP protocol negotiated")?;

    Ok((stream, protocol.to_str()?.to_string()))
}