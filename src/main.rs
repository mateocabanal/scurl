use clap::Parser;
use safenet::{
    app_state::AppState,
    frame::{DataFrame, Frame, InitFrame},
    uuid::Uuid,
    APPSTATE,
};

#[derive(Parser)]
#[command(author, version)]
struct Args {
    url: String,

    #[arg(short, long, default_value = "GET", group = "input")]
    method: String,

    #[arg(short, long, requires = "input")]
    data: Option<String>,

    #[arg(short, long)]
    verbose: bool,
}

#[cfg(any(debug_assertions, feature = "time"))]
use std::time::Instant;

fn main() {

    AppState::init().unwrap();
    let args = Args::parse();

    let log_level = if args.verbose {
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Info
    };
    simple_logger::SimpleLogger::new()
        .with_level(log_level)
        .env()
        .init()
        .unwrap();

    if args.method.as_str() == "POST" && args.data.is_none() {
        log::error!("Cannot send POST request without any data! (use --data)");
    }

    let uuid = APPSTATE.get().unwrap().read().unwrap().uuid.to_string();

    let base_url_split = if let Some(url) = args.url.strip_prefix("http://") {
        url
    } else if let Some(url) = args.url.strip_prefix("https://") {
        url
    } else {
        panic!("no http prefix on url")
    };
    let base_url = base_url_split.split_at(base_url_split.find('/').unwrap()).0;

    let conn_init_url = format!("http://{base_url}/conn/init");
    log::debug!("conn_init url: {conn_init_url}");

    let conn_init_frame = InitFrame::default();
    let conn_init_res = minreq::post(conn_init_url)
        .with_header("s-uuid", &uuid)
        .with_body(conn_init_frame.to_bytes())
        .with_timeout(1000)
        .send();

    if let Ok(res) = conn_init_res {
        #[cfg(any(debug_assertions, feature = "time"))]
        let time = Instant::now();

        if conn_init_frame.from_peer(res.as_bytes()).is_ok() {
            let method = match args.method.to_lowercase().as_str() {
                "get" => minreq::Method::Get,
                "post" => minreq::Method::Post,
                _ => minreq::Method::Get,
            };

            #[cfg(any(debug_assertions, feature = "time"))]
            log::info!("init took {}ms", time.elapsed().as_millis());

            let data_res = match method {
                minreq::Method::Get => minreq::get(args.url).with_header("s-uuid", &uuid).send(),
                minreq::Method::Post => {
                    let mut data_frame = DataFrame::new(args.data.as_ref().unwrap().as_bytes());
                    data_frame
                        .encode_frame(Uuid::from_slice(&res.as_bytes()[3..19]).unwrap())
                        .unwrap();

                    log::trace!("POST data: {:?}", data_frame.body);

                    minreq::post(args.url)
                        .with_body(data_frame.to_bytes())
                        .send()
                }
                _ => minreq::get(args.url).send(),
            };

            #[cfg(any(debug_assertions, feature = "time"))]
            let time = Instant::now();

            log::debug!("data res: {data_res:?}");

            if let Ok(data_res) = data_res {
                let res_data_frame = DataFrame::from_bytes(data_res.as_bytes());

                if let Ok(mut frame) = res_data_frame {
                    frame.decode_frame().unwrap();

                    #[cfg(any(debug_assertions, feature = "time"))]
                    log::info!("data took {}ms", time.elapsed().as_millis());

                    println!("{}", std::str::from_utf8(&frame.body).unwrap());
                } else {
                    println!(
                        "Data received is not a valid Data Frame... printing anyway\n{}",
                        data_res.as_str().unwrap()
                    );
                }
            } else {
                println!("data request failed...");
            }
        } else {
            panic!("Received invalid Initialization Frame!");
        }
    } else {
        panic!("Failed to connect to http://{base_url}/conn/init");
    }
}
