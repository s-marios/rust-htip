use pcap;
use rust_htip;
use rust_htip::Dispatcher;

use std::env;

//Accepts a number of file names
fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    for arg in args {
        print!("opening file {} ...", arg);
        match pcap::Capture::from_file(arg) {
            Ok(capture) => {
                println!("OK");
                parse_captured(capture);
            }
            Err(err) => println!("FAILED! error: {}", err),
        }
    }
}

fn parse_captured<T: pcap::Activated>(mut capture: pcap::Capture<T>) {
    //static setup
    //1. setup our filter (broadcast + lldp)
    capture
        .filter("ether broadcast && ether proto 0x88cc")
        .expect("pcap: unable to set filter");
    //2. get a dispatcher instance
    let mut dispatcher = Dispatcher::new();

    loop {
        let cap_data = capture.next();
        match cap_data {
            Ok(data) => {
                //strip the ethernet header (14 bytes)
                if let Some(htip_frame) = data.get(14..) {
                    let parse_result = dispatcher.parse(htip_frame);
                    match parse_result {
                        Ok(data) => println!("{}\n", data),
                        Err(_err) => println!("skipping bad frame..\n"),
                    }
                }
            }
            //if calling next() causes an error (e.g. no more data), we bail
            Err(_) => break,
        }
    }
}
