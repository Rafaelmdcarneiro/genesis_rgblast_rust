use libftd2xx::{Ftdi, FtdiCommon};
use memmem::{TwoWaySearcher, Searcher};

const READ_BUF_SIZE:usize = 0x200;

pub struct FrameCatcher {
    ft: Ftdi,
    frame_buf: Vec<u8>
}

impl FrameCatcher {

pub fn new() -> Self {
    FrameCatcher {
        ft: Ftdi::new().expect("FTDI not avail"),
        frame_buf: Vec::new(),
    }
}

pub fn catch_single_frame(&mut self) -> Vec<u8> {
    let existing_buf = self.frame_buf.clone();
    let (front, back) = Self::catch_next_end_bytes(&mut self.ft, existing_buf);

    self.frame_buf = back;

    return front;
}

// first vec is before end bytes, second vec is spillover (if any)
fn catch_next_end_bytes(ft:&mut Ftdi, existing_buf:Vec<u8>) -> (Vec<u8>, Vec<u8>) {

    // grab bytes until we find an end marker, then
    // pass that and any spillover back
    let mut read_buf:Vec<u8> = vec![0;READ_BUF_SIZE];
    let mut input_buf:Vec<u8> = existing_buf;
    let byte_pattern:[u8;4] = [0b00000011;4];
    let searcher = TwoWaySearcher::new(&byte_pattern);
    loop {

        // if there's nothing in the queue, don't block? this doesn't really work though...
        match ft.queue_status() {
            Ok(num_bytes) => {
                // println!("Found {} bytes in queue.", num_bytes);
                if num_bytes == 0 {
                    continue;
                }
            },
            Err(e) => {
                panic!("{:?}", e);
            }
        }
        
        // read READ_BUF_SIZE bytes in.
        match ft.read(read_buf.as_mut_slice()) {
            Ok(len) => {
                if len == 0 {
                    continue;
                }
                // println!("Read {} bytes", len);
            },
            Err(e) => {
                panic!("{:?}",e);
            },
        }

        input_buf.extend_from_slice(&read_buf);

        // look for the end sequence in the input_buffer
        match searcher.search_in(&input_buf) {
            Some(index) => {
                // println!("Found end byte sequence at: {}", index);
                // if we find it, see if it landed exactly on a marker on either end
                let remain_bytes = input_buf.len() - (index + 4);

                if remain_bytes == 0 {
                    // println!("Exactly at end of input buffer. Empty vec");
                    return (input_buf, Vec::new());
                } else {
                    let next_buf_start = input_buf.split_off(index+4);

                    return (input_buf, next_buf_start);
                }
            },
            None => (),
        }
    }
}

}

// fn ftdi_input_update(ft:&mut Ftdi) {
//     let image_buffer = catch_single_frame(ft);

//     println!("Retrieved imageBuf of length {}", image_buffer.len());

//     let mut file = File::create("foo.bin").expect("Can't create foo bin");
//     match file.write_all(&image_buffer[..]) {
//         Ok(_) => (),
//         Err(e) => {
//             panic!("{:?}", e);
//         }
//     }
    
// }