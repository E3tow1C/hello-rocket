#[macro_use]
extern crate rocket;
use rocket::{futures::{channel::mpsc, SinkExt, StreamExt}, response::stream::{Event, EventStream}, tokio::{self, time::{interval, sleep, Duration, Instant}}};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/hello/<name>")]
fn echo(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[get("/delay/<seconds>")]
async fn delay(seconds: u64) -> String {
    sleep(Duration::from_secs(seconds)).await;
    format!("Slept for {} seconds", seconds)
}

#[get("/stream/<seconds>")]
async fn stream_delay(seconds: u64) -> EventStream![] {
    let (mut sender, mut receiver) = mpsc::channel::<String>(32);
    let start = Instant::now();

    let total_seconds = seconds;
    let mut interval_timer = interval(Duration::from_millis(500));

    tokio::spawn(async move {
        let mut elapsed_seconds = 0;
        while elapsed_seconds < total_seconds {
            interval_timer.tick().await;
            elapsed_seconds += 1;

            if let Err(_) = sender.send(format!("Elapsed: {} of {} seconds", elapsed_seconds, total_seconds)).await {
                break;
            }
        }

        let _ = sender.send(format!("Complete! Total time: {:?}", start.elapsed())).await;
    });

    EventStream! {
        while let Some(message) = receiver.next().await {
            yield Event::data(message);
        }
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, echo, delay, stream_delay])
}
