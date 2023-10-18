use std::{net::SocketAddr, mem::MaybeUninit, thread, sync::{atomic::{AtomicBool, Ordering}, Arc}, fs::{File, metadata}, io::Read};
use socket2::{Socket, Domain, Type};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // 1. creation
  let socket = Socket::new(Domain::IPV4, Type::STREAM, None).unwrap();
  let addr = "127.0.0.1:8080".parse::<SocketAddr>().unwrap();

  // 2. binding
  socket.bind(&addr.into()).unwrap();

  // 3. listening
  socket.listen(128).unwrap();
  println!("Server listening on {}", addr);


  // 4. accepting connections
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

    handle_connection(s);
  }
}

fn handle_connection(s: Socket) {
  let buffer = [MaybeUninit::uninit(); 1024];
  let running = Arc::new(AtomicBool::new(true));
  let running_clone = running.clone();

  let thread_handle = thread::spawn(move || {
    while running_clone.load(Ordering::Relaxed) {
      let (s, received_text) = handle_client_data(&s, buffer);

      if received_text == "Sair" {
        s.send("Conexão finalizada.".as_bytes()).expect("failed to send message to clients");
        running_clone.store(false, Ordering::Relaxed);
      } 
      else {
        s.send(received_text.as_bytes()).expect("failed to send test to clients");
        println!("received: {:?}", received_text);
      }
    }
  });

  thread_handle.join().expect("Thread panicked");
}

fn handle_client_data(s: &Socket, mut buffer: [MaybeUninit<u8>; 1024]) -> (&Socket, &'static str) {
  let (size, _) =  match s.recv_from(&mut buffer) {
    Ok(s) => s,
    Err(e) => {
      eprintln!("Failed to receive data from client: {}", e);
      return (s, "0");
    }
  };
  
  let received_data = unsafe {
    std::slice::from_raw_parts(buffer.as_ptr() as *const u8, size)
  };
  
  let received_text = match std::str::from_utf8(received_data) {
    Ok(s) => s.trim_end(),
    Err(e) => {
      eprintln!("Failed to format data: {}", e);
      ""
    }
  };

  (s, received_text)
}
