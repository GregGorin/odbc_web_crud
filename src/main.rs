use async_std::fs;
use async_std::net::TcpListener;
use async_std::net::TcpStream;
use async_std::prelude::*;
// use async_std::task;
use futures::stream::StreamExt;
use local_ip_address::local_ip;
use std::net::{IpAddr, SocketAddr};
use std::u32;
// use std::time::Duration;

mod odbc;
use odbc::*;
struct MyRequest {
    headers: Vec<String>,
    body: Option<String>,
}
impl MyRequest {
    pub fn pass_key_is_ok(&self) -> bool {
        match self.get_header_parametr("PassKey") {
            Some(passkey) => correct(passkey, self.body_length()),
            None => false,
        }
    }
    fn body_length(&self) -> u32 {
        self.body.as_ref().unwrap().len() as u32
    }
    pub fn get_header_parametr(&self, parametr: &str) -> Option<String> {
        for header_line in &self.headers {
            if header_line.contains(parametr) {
                return Some(
                    header_line.split(":").collect::<Vec<&str>>()[1]
                        .trim()
                        .to_string(),
                );
            }
        }
        None
    }
}
fn correct(passkey: String, cont_length: u32) -> bool {
    // let mut sum = cont_length;
    // for ch in passkey.chars() {
    //     sum = sum + ch.to_digit(10).unwrap_or(0);
    // }
    // sum % 7 == 0
    true
}

fn request(request_txt: String) -> MyRequest {
    let mut next_body = false;
    let mut headers: Vec<String> = Vec::new();
    let mut body = String::new();

    for line in request_txt.lines() {
        if next_body {
            body.push_str(line)
        } else {
            if line == "" {
                next_body = true;
            } else {
                headers.push(line.to_string());
            }
        }
    }
    MyRequest {
        headers,
        body: match body.len() {
            0 => None,
            _ => Some(body),
        },
    }
}

#[async_std::main]
async fn main() {
    let local_ip = local_ip().unwrap();

    let socket = match local_ip {
        IpAddr::V4(ip) => SocketAddr::from((ip.octets(), 7878)),
        IpAddr::V6(ip) => SocketAddr::from((ip.octets(), 7878)),
    };

    let listener = TcpListener::bind(socket).await.unwrap();
    println!("Server start on : {}", socket);

    listener
        .incoming()
        .for_each_concurrent(/* limit */ None, |tcpstream| async move {
            let tcpstream = tcpstream.unwrap();
            handle_connection(tcpstream).await;
        })
        .await;
}
async fn read_http_request(stream: &mut TcpStream) -> Vec<u8> {
    let mut vec_u8: Vec<u8> = Vec::new();
    loop {
        let mut buffer = [0; 1024];
        let mut bottom = false;
        stream.read(&mut buffer).await.unwrap();
        for value in buffer {
            if value == 0 {
                bottom = true;
                break;
            } else {
                vec_u8.push(value)
            }
        }
        if bottom {
            break;
        };
    }
    vec_u8
}

async fn handle_connection(mut stream: TcpStream) {
    let vec_u8 = read_http_request(&mut stream).await;
    let request = request(String::from_utf8_lossy(&vec_u8).to_string());
    let protocol = &request.headers[0];
    // println!("Protocol: {:?}", protocol);

    let http_200 = "HTTP/1.1 200 OK\r\n\r\n";
    let http_404 = "HTTP/1.1 404 NOT FOUND\r\n\r\n";

    let response = match protocol.as_str() {
        "GET / HTTP/1.1" => {
            let contents = fs::read_to_string("hello.html").await.unwrap();
            format!("{http_200}{contents}")
        }
        "POST / HTTP/1.1" => {
            if request.pass_key_is_ok() && request.body.is_some() {
                match serde_json::from_str(&request.body.unwrap()) {
                    Ok(job) => {
                        let res = execute_job(job);
                        let contents = serde_json::to_string(&res).unwrap();
                        format!("{http_200}{contents}")
                    }
                    Err(e) => {
                        // let contents = fs::read_to_string("404.html").await.unwrap();
                        format!("{http_404}{e}")
                    }
                }
            } else {
                let contents = fs::read_to_string("404.html").await.unwrap();
                format!("{http_404}{contents}")
            }
        }
        _ => {
            let contents = fs::read_to_string("404.html").await.unwrap();
            format!("{http_404}{contents}")
        }
    };

    stream.write(response.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write};

    use super::odbc::*;

    #[test]
    fn main() {
        let job = Job {
            odbc_source: String::from("Driver={Microsoft Access Driver (*.mdb, *.accdb)};DBQ=C:\\BIN\\work_space\\msaccess\\Database1.accdb"),
            sql_text: String::from("select * from table1"),
            data_set: vec![vec!["".to_string()]],
        };

        let res = execute_job(job);

        println!("{:?}", res);
    }

    // Convert the Point to a JSON string.
    #[test]
    fn save_to_json() {
        let job = Job {
            odbc_source: String::from("Driver={Microsoft Access Driver (*.mdb, *.accdb)};DBQ=C:\\BIN\\work_space\\msaccess\\Database1.accdb"),
            sql_text: String::from("select * from table1"),
            data_set: vec![vec!["".to_string()]],
        };

        let serialized = serde_json::to_string(&job).unwrap();
        let path = "data.json";
        let mut f = File::create(path).expect("Ошибка создания файла!");
        f.write_all(serialized.as_bytes())
            .expect("Не удалось записать!");
    }
}
