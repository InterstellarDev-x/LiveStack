use redis::streams::StreamMaxlen;
use redis::{Commands, RedisResult};

const BETTERUPTIME: &str = "better-uptime";
const STREAMS: &[&str] = &[BETTERUPTIME];

pub mod config;

pub fn redis_main() {
    let client = redis::Client::open("redis://127.0.0.1/").expect("client");
    println!("Demonstrating XADD followed by XREAD, single threaded\n");
    add_records(&client).expect("contrived record generation");
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
            maxlen, // how many latest entries should we keep in redis while adding
            "*",
            &[
                ("url", String::from("www.google.com")),
                ("url", "www.facebook.com".into()),
            ],
        )?;
    }

    let len: usize = con.xlen(BETTERUPTIME).unwrap();

    // println!("{}" , con.xlen::<_, usize>(STREAMS[0]).unwrap());
    println!("thie size is {}", len);

    Ok(())
}

/// Block the thread for this many milliseconds while
/// waiting for data to arrive on the stream.

/// Read back records from all three streams, if they're available.
/// Doesn't bother with consumer groups.  Generally the user
/// would be responsible for keeping track of the most recent
/// ID from which they need to read, but in this example, we
/// just go back to the beginning of time and ask for all the
/// records in the stream.

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
