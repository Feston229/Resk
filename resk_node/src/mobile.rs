// Mobile specific code
use crate::controllers::FLUTTER_UDP_PORT;
use std::error::Error;
use std::net::UdpSocket;

pub struct MobileClipboard {
    socket: UdpSocket,
    port: i32,
}

impl MobileClipboard {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let port = FLUTTER_UDP_PORT.read().await;
        Ok(MobileClipboard {
            socket: UdpSocket::bind("127.0.0.1:0")?,
            port: *port,
        })
    }
    fn send_udp_request_flutter(
        &mut self,
        request: String,
    ) -> Result<String, Box<dyn Error>> {
        let server_addr = format!("127.0.0.1:{}", self.port);
        self.socket.send_to(request.as_bytes(), server_addr)?;
        let mut buf = [0; 4096];
        self.socket.recv_from(&mut buf)?;
        let buf = String::from_utf8_lossy(&buf);
        Ok(buf.to_string())
    }
    pub fn get_contents(&mut self) -> Result<String, Box<dyn Error>> {
        let response =
            self.send_udp_request_flutter("get_content:".to_string())?;
        log::info!("Get content from flutter: {response}");
        Ok(response)
    }
    // Send udp request to get it from flutter
    pub fn set_contents(
        &mut self,
        content: String,
    ) -> Result<(), Box<dyn Error>> {
        if self.send_udp_request_flutter(format!("set_content:{}", content))?
            != "OK"
        {
            log::error!("Failed to send clipboard to flutter");
        } else {
            log::info!("Sent clipboard to flutter");
        }
        Ok(())
    }
}
