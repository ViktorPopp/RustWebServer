use web_server::ThreadPool;
use std::{
    collections::HashMap,
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};
use rand::{distributions::Alphanumeric, Rng};

#[derive(Debug)]
struct UrlShortener {
    map: HashMap<String, String>,
}

impl UrlShortener {
    fn new() -> Self {
        UrlShortener {
            map: HashMap::new(),
        }
    }

    fn generate_alias(&self) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect()
    }

    fn shorten(&mut self, long: String) -> String {
        let short = self.generate_alias();
        self.map.insert(short.clone(), long);
        short
    }

    fn resolve(&self, short: &str) -> Option<&String> {
        self.map.get(short)
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);
    let shortener = Arc::new(Mutex::new(UrlShortener::new()));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let shortener = Arc::clone(&shortener);

        pool.execute(move || {
            handle_connection(stream, shortener);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream, shortener: Arc<Mutex<UrlShortener>>) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("");

    let response = if method == "GET" {
        let short_code = path.trim_start_matches('/');
        let shortener = shortener.lock().unwrap();
        
        if let Some(url) = shortener.resolve(short_code) {
            format!("HTTP/1.1 302 Found\r\nLocation: {}\r\n\r\n", url)
        } else if short_code.is_empty() {
            let contents = fs::read_to_string("www/index.html").unwrap();
            format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}", contents.len(), contents)
        } else {
            format!("HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n")
        }
    } else if method == "POST" && path == "/shorten" {
        let body = buf_reader.lines().collect::<Result<Vec<_>, _>>().unwrap().join("\n");
        let long_url = body.split('=').nth(1).unwrap_or("").to_string();
        
        let mut shortener = shortener.lock().unwrap();
        let short = shortener.shorten(long_url);
        
        format!("HTTP/1.1 201 Created\r\nContent-Length: {}\r\n\r\n{}", short.len(), short)
    } else {
        format!("HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\n\r\n")
    };

    stream.write_all(response.as_bytes()).unwrap();
}
