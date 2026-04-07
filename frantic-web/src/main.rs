use frantic_client::{CrDocument, FranticClient};
use frantic_core::cr::Cr;
use gloo::console;
use web_sys::HtmlInputElement;
use yew::{Component, Context, Html, TargetCast, html};

pub enum Msg {
    Cr(CrDocument),
    Search(String),
}

pub struct App {
    cr: Cr<'static>,
    date: String,
    query: String,
}
#[cfg(debug_assertions)]
async fn fetch_cr() -> Msg {
    Msg::Cr(CrDocument {
        text: include_str!("../../docs/MagicCompRules 20260227.txt").into(),
        date: "TESTING".into(),
    })
}

#[cfg(not(debug_assertions))]
async fn fetch_cr() -> Msg {
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
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(fetch_cr());
        Self {
            cr: Cr::default(),
            date: String::new(),
            query: String::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Cr(cr) => {
                self.date = cr.date;
                self.cr = Cr::parse(String::leak(cr.text));
            }
            Msg::Search(query) => {
                self.query = query;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let words: Vec<String> = self.query.split_whitespace().map(String::from).collect();
        let displayed = if words.is_empty() {
            self.cr.clone()
        } else {
            self.cr.search(&words)
        };

        let oninput = ctx.link().callback(|e: yew::events::InputEvent| {
            let input = e.target_unchecked_into::<HtmlInputElement>();
            Msg::Search(input.value())
        });

        html! {
            <div class="cr-container">
                <h1 class="cr-title">{ format!("Comprehensive Rules as of {}", self.date) }</h1>
                <input
                    class="cr-search"
                    type="text"
                    placeholder="Search rules..."
                    value={ self.query.clone() }
                    {oninput}
                />
                { for displayed.0.iter().map(|section| html! {
                    <div class="cr-section">
                        <h2 class="cr-section-header">{ section.text }</h2>
                        { for section.subsections.iter().map(|subsection| html! {
                            <div class="cr-subsection">
                                <h3 class="cr-subsection-header">{ subsection.text }</h3>
                                { for subsection.rules.iter().map(|rule| html! {
                                    <div class="cr-rule">
                                        <p class="cr-rule-text">{ rule.text }</p>
                                        { for rule.subrules.iter().map(|subrule| html! {
                                            <p class="cr-subrule-text">{ subrule.text }</p>
                                        }) }
                                    </div>
                                }) }
                            </div>
                        }) }
                    </div>
                }) }
            </div>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
