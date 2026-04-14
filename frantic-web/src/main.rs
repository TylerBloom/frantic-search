use frantic_client::CrDocument;
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

async fn fetch_cr() -> Msg {
    const CACHE_KEY_TEXT: &str = "frantic_cr_text";
    const CACHE_KEY_DATE: &str = "frantic_cr_date";
    const CACHE_KEY_TIME: &str = "frantic_cr_cached_at";
    const CACHE_DURATION_MS: f64 = 12.0 * 60.0 * 60.0 * 1000.0;
    fn get_cached_cr() -> Option<frantic_client::CrDocument> {
        let storage = web_sys::window()?.local_storage().ok()??;
        let cached_at: f64 = storage.get_item(CACHE_KEY_TIME).ok()??.parse().ok()?;
        if js_sys::Date::now() - cached_at > CACHE_DURATION_MS {
            return None;
        }
        let text = storage.get_item(CACHE_KEY_TEXT).ok()??;
        let date = storage.get_item(CACHE_KEY_DATE).ok()??;
        Some(frantic_client::CrDocument { text, date })
    }

    fn set_cached_cr(cr: &frantic_client::CrDocument) {
        let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok()?) else {
            return;
        };
        let now = js_sys::Date::now();
        let _ = storage.set_item(CACHE_KEY_TIME, &now.to_string());
        let _ = storage.set_item(CACHE_KEY_TEXT, &cr.text);
        let _ = storage.set_item(CACHE_KEY_DATE, &cr.date);
    }

    if let Some(cr) = get_cached_cr() {
        return Msg::Cr(cr);
    }
    let client = frantic_client::FranticClient::connect();
    let cr = match client.fetch_latest_indirect().await {
        Ok(cr) => cr,
        Err(err) => {
            gloo::console::log!(err.to_string());
            panic!()
        }
    };
    set_cached_cr(&cr);
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
