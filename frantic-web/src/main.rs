
use frantic_client::{CrDocument, FranticClient};
use gloo::console::{self};
use js_sys::Date;
use yew::{Component, Context, Html, html};

// Define the possible messages which can be sent to the component
pub enum Msg {
    Increment,
    Decrement,
    Cr(CrDocument),
}

pub struct App {
    value: i64, // This will store the counter value
    // client: FranticClient<ReadOnly>,
    cr: CrDocument,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let client = FranticClient::connect();
        ctx.link().send_future(async move {
            console::log!("Fetching latest CR...");
            let cr = client.fetch_latest().await.unwrap();
            console::log!(format!("Cr fetched: {cr:?}"));
            Msg::Cr(cr)
        });
        Self {
            value: 0,
            cr: CrDocument::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Increment => {
                self.value += 1;
                console::log!("plus one"); // Will output a string to the browser console
                true // Return true to cause the displayed change to update
            }
            Msg::Decrement => {
                self.value -= 1;
                console::log!("minus one");
                true
            }
            Msg::Cr(cr) => {
                self.cr = cr;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <div class="panel">
                    // A button to send the Increment message
                    <button class="button" onclick={ctx.link().callback(|_| Msg::Increment)}>
                        { "+1" }
                    </button>

                    // A button to send the Decrement message
                    <button onclick={ctx.link().callback(|_| Msg::Decrement)}>
                        { "-1" }
                    </button>

                    // A button to send two Increment messages
                    <button onclick={ctx.link().batch_callback(|_| vec![Msg::Increment, Msg::Increment])}>
                        { "+1, +1" }
                    </button>

                </div>

                // Display the current value of the counter
                <p class="counter">
                    { self.value }
                </p>

                // Display the current date and time the page was rendered
                <p class="footer">
                    { "Rendered: " }
                    { String::from(Date::new_0().to_string()) }
                </p>

                <p> { format!("{:#?}", self.cr) } </p>
            </div>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
