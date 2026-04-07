use frantic_client::{CrDocument};
use frantic_core::cr::Cr;
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
    let client = frantic_client::FranticClient::connect();
    gloo::console::log!("Fetching latest CR...");
    let cr = match client.fetch_latest().await {
        Ok(cr) => cr,
        Err(err) => {
            gloo::console::log!(err.to_string());
            panic!()
        }
    };
    console::log!(format!("Cr fetched: {cr:?}"));
    Msg::Cr(cr)
}

/// Renders `text` as HTML, wrapping every occurrence of each word in a `<mark>`.
fn highlight(text: &str, words: &[String]) -> Html {
    if words.is_empty() {
        return html! { {text} };
    }

    // Collect all (start, end) byte ranges that match any word.
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for word in words {
        if word.is_empty() {
            continue;
        }
        let mut cursor = 0;
        while let Some(pos) = text[cursor..].find(word.as_str()) {
            let start = cursor + pos;
            let end = start + word.len();
            ranges.push((start, end));
            cursor = end;
        }
    }

    if ranges.is_empty() {
        return html! { {text} };
    }

    // Sort then merge overlapping/adjacent ranges.
    ranges.sort_by_key(|r| r.0);
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (start, end) in ranges {
        match merged.last_mut() {
            Some(last) if start <= last.1 => last.1 = last.1.max(end),
            _ => merged.push((start, end)),
        }
    }

    // Build fragments: plain text interleaved with <mark> spans.
    let mut parts: Vec<Html> = Vec::new();
    let mut cursor = 0;
    for (start, end) in merged {
        if cursor < start {
            let before = &text[cursor..start];
            parts.push(html! { {before} });
        }
        let matched = &text[start..end];
        parts.push(html! { <mark class="cr-highlight">{matched}</mark> });
        cursor = end;
    }
    if cursor < text.len() {
        parts.push(html! { {&text[cursor..]} });
    }

    html! { <>{ for parts.into_iter() }</> }
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
                        <h2 class="cr-section-header">{ highlight(section.text, &words) }</h2>
                        { for section.subsections.iter().map(|subsection| html! {
                            <div class="cr-subsection">
                                <h3 class="cr-subsection-header">{ highlight(subsection.text, &words) }</h3>
                                { for subsection.rules.iter().map(|rule| html! {
                                    <div class="cr-rule">
                                        <p class="cr-rule-text">{ highlight(rule.text, &words) }</p>
                                        { for rule.subrules.iter().map(|subrule| html! {
                                            <p class="cr-subrule-text">{ highlight(subrule.text, &words) }</p>
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
