use std::{str, net::SocketAddr, mem::MaybeUninit, thread, sync::{Arc, atomic::{AtomicBool, Ordering}}, fs::File, io::{Read, Write}};
use socket2::{Socket, Domain, Type};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket = Socket::new(Domain::IPV4, Type::STREAM, None).unwrap();
    let addr = "127.0.0.1:8080".parse::<SocketAddr>().unwrap();
    socket.bind(&addr.into()).unwrap();
    socket.listen(128).unwrap();
    println!("Server listening on {}", addr);

    loop {
        let (s, _) = match socket.accept() {
            Ok(s) => {
                println!("{:?}, connected", s.1.as_socket_ipv4().unwrap());
                s
            },
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
                continue;
            }
        };

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        thread::spawn(move || {
            handle_connection(s, running_clone);
        });
    }
}

fn handle_connection(mut s: Socket, running: Arc<AtomicBool>) {
    let mut buffer = [0; 1024];

    while running.load(Ordering::Relaxed) {
        if let Ok(size) = s.read(&mut buffer) {
            let request = str::from_utf8(&buffer[..size]).unwrap();
            let response = generate_response(request);
            
            s.write_all(&response).expect("Failed to send response to the client.");
        } else {
            break;
        }
    }
}

fn generate_response(request: &str) -> Vec<u8> {
  // Parse the HTTP request
  let lines: Vec<&str> = request.split("\r\n").collect();
  let first_line: &str = lines[0];
  let parts: Vec<&str> = first_line.split_whitespace().collect();
  let method = parts[0];
  let path = parts[1];

  let file_path = format!("{}{}", "src/files", path);

  if method == "GET" {
      if path.contains(".html") {
          if let Ok(mut file) = File::open(file_path) {
              let mut contents = Vec::new();
              file.read_to_end(&mut contents).expect("Failed to read the file.");
              let response = format!(
                  "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                  contents.len(),
                  str::from_utf8(&contents).unwrap()
              );
              return response.into_bytes();
          } else {
              // Handle 404 Not Found
              return b"HTTP/1.1 404 Not Found\r\n\r\n<h1>404 Not Found</h1>".to_vec();
          }
      } else if path.contains(".jpg") || path.contains(".jpeg") {
          if let Ok(mut file) = File::open(file_path) {
              let mut contents = Vec::new();
              file.read_to_end(&mut contents).expect("Failed to read the file.");
              let response = format!(
                  "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: image/jpeg\r\n\r\n",
                  contents.len()
              );
              let mut response_bytes = response.into_bytes();
              response_bytes.extend_from_slice(&contents);
              return response_bytes;
          } else {
              return b"HTTP/1.1 404 Not Found\r\n\r\n<h1>404 Not Found</h1>".to_vec();
          }
      }
  }
  // Handle other HTTP methods or requests
  return b"HTTP/1.1 501 Not Implemented\r\n\r\n<h1>501 Not Implemented</h1>".to_vec();
}
