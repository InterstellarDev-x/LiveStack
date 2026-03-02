use redis::streams::{self, StreamId, StreamKey, StreamMaxlen, StreamReadOptions, StreamReadReply};
use redis::{Commands, RedisResult, Value};
use std::time::{SystemTime, UNIX_EPOCH};
const BETTERUPTIME: &str = "better-uptime";
const STREAMS: &[&str] = &[BETTERUPTIME];

pub mod config;
pub mod pool;



pub fn redis_main() {
    let client = redis::Client::open("redis://127.0.0.1/").expect("client");
    println!("Demonstrating XADD followed by XREAD, single threaded\n");
    add_records(&client).expect("contrived record generation");
    read_records(&client).expect("simple read");
    // demo_group_reads(&client);
    clean_up(&client)
}


pub fn add_records(client: &redis::Client) -> RedisResult<()> {
    let mut con = client.get_connection().expect("conn");

    let maxlen = StreamMaxlen::Approx(1000);

    // a stream whose records have two fields
    for _ in 0..1 {
        let _: () = con.xadd_maxlen(
            BETTERUPTIME,
            maxlen , // how many latest entries should we keep in redis while adding 
            "*",
            &[("url", String::from("www.google.com")), ("url", "www.facebook.com".into())],
        )?;
    }
      for _ in 0..1 {
        let _: () = con.xadd_maxlen(
            BETTERUPTIME,
            maxlen , // how many latest entries should we keep in redis while adding 
            "*",
            &[("url", String::from("www.google.com")), ("url", "www.facebook.com".into())],
        )?;
    }

    let len : usize  = con.xlen(BETTERUPTIME).unwrap();

    // println!("{}" , con.xlen::<_, usize>(STREAMS[0]).unwrap());
    println!("thie size is {}" , len);

    Ok(())
}



/// Generate a potentially unique value.
fn arbitrary_value() -> String {
    format!(
        "{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time travel")
            .as_nanos()
    )
}


/// Block the thread for this many milliseconds while
/// waiting for data to arrive on the stream.
const BLOCK_MILLIS: usize = 5000;

/// Read back records from all three streams, if they're available.
/// Doesn't bother with consumer groups.  Generally the user
/// would be responsible for keeping track of the most recent
/// ID from which they need to read, but in this example, we
/// just go back to the beginning of time and ask for all the
/// records in the stream.
fn read_records(client: &redis::Client) -> RedisResult<()> {
    let mut con = client.get_connection().expect("conn");

    let opts = StreamReadOptions::default().block(BLOCK_MILLIS);

    // Oldest known time index
    let starting_id = "0-0";
    // Same as above
    let another_form = "0";

    let srr: StreamReadReply = con
        .xread_options(STREAMS, &[starting_id, another_form, starting_id], &opts)
        .expect("read");

    for StreamKey { key, ids } in srr.keys {
        println!("Stream {key}");
        for StreamId { id, map, .. } in ids {
            println!("\tID {id}");
            for (n, s) in map {
                if let Value::BulkString(bytes) = s {
                    println!("\t\t{}: {}", n, String::from_utf8(bytes).expect("utf8"))
                } else {
                    panic!("Weird data")
                }
            }
        }
    }

    Ok(())
}



const GROUP_NAME: &str = "example-group-aaa";


fn clean_up(client: &redis::Client) {
    let mut con = client.get_connection().expect("con");
    for k in STREAMS {
        let trimmed: RedisResult<()> = con.xtrim(*k, StreamMaxlen::Equals(0));
        trimmed.expect("trim");

        let destroyed: RedisResult<()> = con.xgroup_destroy(*k, GROUP_NAME);
        destroyed.expect("xgroup destroy");
    }
}