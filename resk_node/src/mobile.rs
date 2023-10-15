// Mobile specific code
use std::error::Error;
use std::net::UdpSocket;

pub struct MobileClipboard {
    socket: UdpSocket,
    port: i32,
}

impl MobileClipboard {
    pub fn new(port: i32) -> Result<Self, Box<dyn Error>> {
        Ok(MobileClipboard {
            socket: UdpSocket::bind("127.0.0.1:0")?,
            port,
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
        _contents: String,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
