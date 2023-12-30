use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};
use websvr::{HttpRequest, ThreadPool};

fn main() {
    let addr = "0.0.0.0:7878";
    let listener = match TcpListener::bind(addr) {
        Ok(item) => item,
        Err(_) => panic!("无法绑定到{},请检查IP及端口是否可用.", addr),
    };
    let pool = ThreadPool::new(5);
    for stream in listener.incoming().take(10) {
        match stream {
            Ok(s) => pool.execute(|| {
                handle_connection(s);
            }),
            Err(_) => continue,
        };
    }
    println!("服务器关闭中...");
}
fn handle_connection(stream: TcpStream) {
    let buffer_reader = BufReader::new(&stream);
    let http_request: Vec<_> = buffer_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    let request_line = http_request.get(0);
    match request_line {
        Some(req) => {
            let http_req = HttpRequest::new(req.to_string());
            match http_req.method {
                websvr::Method::Post => {}
                websvr::Method::Get => {
                    write_response(stream, &http_req.url);
                }
            }
        }
        None => return,
    }
}
fn write_response(mut stream: TcpStream, path: &str) {
    let (code, file_path) = match fs::metadata(path) {
        Ok(_) => ("200 OK", path),
        Err(_) => ("404 NOT FOUND", "404.html"),
    };
    stream
        .write_all(get_response(code, file_path).as_bytes())
        .unwrap();
}
fn get_response(code: &str, path: &str) -> String {
    let status_line = format!("HTTP/1.1 {}", code);
    let contents = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => String::from("File not supported!"),
    };
    let length = contents.len();
    format!(
        "{}\r\nContent-Length:{}\r\n\r\n{}",
        status_line, length, contents
    )
}
