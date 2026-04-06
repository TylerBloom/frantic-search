use frantic_client::{CrDocument, FranticClient};
use frantic_core::cr::Cr;
use gloo::console::{self};
use yew::{Component, Context, Html, html};

// Define the possible messages which can be sent to the component
pub enum Msg {
    Cr(CrDocument),
}

pub struct App {
    cr: Cr<'static>,
    date: String,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async move {
            let client = FranticClient::connect();
            console::log!("Fetching latest CR...");
            let cr = match client.fetch_latest().await {
                Ok(cr) => cr,
                Err(err) => {
                    console::log!(err.to_string());
                    panic!()
                }
            };
            console::log!(format!("Cr fetched: {cr:?}"));
            Msg::Cr(cr)
        });
        Self {
            cr: Cr::default(),
            date: String::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Cr(cr) => {
                self.date = cr.date;
                self.cr = Cr::parse(String::leak(cr.text));
            }
        }
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <p> { format!("Rules as of {}", self.date) } </p>
                <p> { self.cr.to_string() } </p>
            </div>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
