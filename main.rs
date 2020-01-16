extern crate threadpool;
extern crate rand;
use threadpool::ThreadPool;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::env;
use std::str;
use std::io;


fn get_session_key() -> String {
  use rand::Rng;
  const CHARSET: &[u8] = b"123456789";
    const LEN: usize = 10;
    let mut rng = rand::thread_rng();

    let session_key: String = (0..LEN)
        .map(|_| {
            let idx = rng.gen_range(0, CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    //println!("{}",session_key);
  session_key
}

fn get_hash_str() -> String {
  use rand::Rng;
  const CHARSET: &[u8] = b"123456";
    const LEN: usize = 5;
    let mut rng = rand::thread_rng();

    let hash: String = (0..LEN)
        .map(|_| {
            let idx = rng.gen_range(0, CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    //println!("{}",hash);
  hash
}

fn next_session_key(hash: &String, key: &String) -> String {
    if hash == "" {
        println!("Hash code is empty");
    }
    let mut result: i64 = 0;
    let mut i: i64 = 0;
    let mut h: i64 = hash.parse().unwrap();
    while i < 5 {
        i = i + 1;
        result = result + calc_hash(&key,h%10);
        h = h/10;
    }
    if result > 9999999999 {
        result = result / 10;
    }
    let mut res: String = result.to_string();
    if result < 10 {
        let conc = "000000000";
        res = format!("{}{}", conc, res);
    } else if result < 100 {
        let conc = "00000000";
        res = format!("{}{}", conc, res);
    } else if result < 1000 {
        let conc = "0000000";
        res = format!("{}{}", conc, res);
    } else if result < 10000 {
        let conc = "000000";
        res = format!("{}{}", conc, res);
    } 
    res
}

fn calc_hash(key: &String, val: i64) -> i64 {
    let mut result :i64 = 0;
    let res1: i64 = key.parse().unwrap();
    if val == 1{
        let k1 :i64 = res1/100000;
        result = k1%97;
    } else if val == 2{
        let mut i :i64 = 0;
        let mut k2 :i64 = res1;
        while i < 10 {
            i = i + 1;
            result = result*10+(k2%10);
            k2 = k2/10;
        }
    } else if val == 3{
        let k31 :i64 = res1/100000;
        let k32 :i64 = res1%100000;
        result = k32*100000 + k31;
    } else if val == 4{
        let mut i :i64 = 0;
        let mut k4 :i64 = res1;
        while i < 8 {
            i = i + 1;
            k4 = k4/10;
            result = result+(k4%10)+41;
        }
    } else if val == 5{
        let num_string = key.to_string();
        let mut i = 0;
        while i < 9 {
            i = i + 1;
            let b: u8 = num_string.as_bytes()[i];
            let c: u8 = b ^ 43;
            let k5: i64 = c as i64;
            result = result + k5;
        }
    } else {
        result = res1 + val;
    }
    result
}


fn handle_client(mut stream: TcpStream) {
	let mut hash_from_client: String;
	let mut key_from_client: String;
	let mut next_key: String;
	loop {
		let mut data = [0 as u8; 32]; // using 32 byte buffer
		match stream.read(&mut data) {
			Ok(size) => {
				hash_from_client = str::from_utf8(&data[0..5]).unwrap().to_string();
				key_from_client = str::from_utf8(&data[5..15]).unwrap().to_string();
				next_key = next_session_key(&hash_from_client, &key_from_client);
				let text = str::from_utf8(&data[15..size-2]).unwrap().to_string();
				let key_text = format!("{}{}",next_key,text);
				let msg = key_text.into_bytes();
				println!("key from client: {:?} next key: {:?} msg from client: {:?}"
				,key_from_client, next_key, str::from_utf8(&data[15..size-2]).unwrap());
				stream.write(&msg).unwrap();
			},
			Err(_) => {
				println!("An error occurred, terminating connection");
				stream.shutdown(Shutdown::Both).unwrap();
			}
		} 
	}
}

fn main() {
	let args: Vec<String> = env::args().collect();
    let port = &args[1];
    let ip = &args[2];
	let n = &args[3];
	if ip == "0.0.0.0" {
		let kolvo: usize = n.parse().unwrap();
		let host = format!("{}:{}", ip, port);
		let pool = ThreadPool::new(kolvo);
		let listener = TcpListener::bind(host).unwrap();
		// accept connections and process them, spawning a new thread for each one
		println!("Server listening on port {}", port);
		for stream in listener.incoming() {
			match stream {
				Ok(stream) => {
					println!("New connection");
					pool.execute(move|| {
						// connection succeeded
						handle_client(stream)
					});
				}
				Err(e) => {
					println!("Error: {}", e);
					/* connection failed */
				}
			}
		}
		// close the socket server
		drop(listener);
	} else {
		let hash_to_server: String = get_hash_str();
		let mut key_to_server: String;
		let mut key_to_check: String;
		let mut key_from_server: String;
		let host = format!("{}:{}", ip, port);
		match TcpStream::connect(host) {
			Ok(mut stream) => {
				println!("Successfully connected to server on port {}", port);
				loop {
					key_to_server = get_session_key();
					key_to_check = next_session_key(&hash_to_server, &key_to_server);
					let keys = format!("{}{}", hash_to_server, key_to_server);
					let mut input_text = String::new();
					io::stdin().read_line(&mut input_text).expect("failed to read from stdin");
					let keys_text = format!("{}{}",keys,input_text);
					let msg = keys_text.into_bytes();
					stream.write(&msg).unwrap();
					println!("Sent msg, awaiting reply...");
	
					let mut data = [0 as u8; 32]; // using 32 byte buffer
					match stream.read(&mut data) {
						Ok(size) => {
								key_from_server = str::from_utf8(&data[0..10]).unwrap().to_string();
								if key_from_server != key_to_check {break}
								println!("Key from server: {:?} key to check: {:?} Message from server: {:#?}"
								,key_from_server, key_to_check, str::from_utf8(&data[10..size]).unwrap());
						},
						Err(e) => {
							println!("Failed to receive data: {}", e);
						}
					}
				}
			},
			Err(e) => {
				println!("Failed to connect: {}", e);
			}
		}
		println!("Terminated.");
	}
}