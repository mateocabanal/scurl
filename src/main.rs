use clap::Parser;
use safenet::{
    frame::{DataFrame, Frame, InitFrame},
    uuid::Uuid,
};

#[derive(Parser)]
#[command(author, version)]
struct Args {
    url: String,

    #[arg(short, long, default_value = "GET", group = "input")]
    method: String,

    #[arg(short, long, requires = "input")]
    data: Option<String>,
}

fn main() {
    let args = Args::parse();

    if args.method.as_str() == "POST" && args.data.is_none() {
        panic!("Cannot send POST request without any data! (use --data)");
    }

    let base_url_split = args.url.split('/').collect::<Vec<&str>>();
    let base_url =
        base_url_split.first().unwrap().to_string() + "//" + *base_url_split.get(2).unwrap();

    let conn_init_url = format!("{base_url}/conn/init");

    let conn_init_frame = InitFrame::default();
    let conn_init_res = minreq::post(conn_init_url)
        .with_body(conn_init_frame.to_bytes())
        .with_timeout(1000)
        .send();

    if let Ok(res) = conn_init_res {
        if conn_init_frame.from_peer(res.as_bytes()).is_ok() {
            let method = match args.method.to_lowercase().as_str() {
                "get" => minreq::Method::Get,
                "post" => minreq::Method::Post,
                _ => minreq::Method::Get,
            };

            let data_res = match method {
                minreq::Method::Get => minreq::get(args.url).send(),
                minreq::Method::Post => {
                    let mut data_frame = DataFrame::new(args.data.as_ref().unwrap().as_bytes());
                    data_frame
                        .encode_frame(Uuid::from_slice(&res.as_bytes()[3..19]).unwrap())
                        .unwrap();
                    minreq::post(args.url)
                        .with_body(data_frame.to_bytes())
                        .send()
                }
                _ => minreq::get(args.url).send(),
            };

            if let Ok(data_res) = data_res {
                let mut res_data_frame = DataFrame::from_bytes(data_res.as_bytes()).unwrap();
                res_data_frame.decode_frame().unwrap();

                println!("{}", std::str::from_utf8(&res_data_frame.body).unwrap());
            } else {
                println!("data request failed...");
            }
        }
    } else {
        conn_init_res.unwrap();
    }
}
