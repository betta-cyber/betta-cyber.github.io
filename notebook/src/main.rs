use yew::{html, Callback, ClickEvent, Component, ComponentLink, Html, ShouldRender};
use yew::services::fetch::{Request, Response, FetchService};
use yew::format::{Json, Nothing};
use failure::Error;

struct App {
    clicked: bool,
    onclick: Callback<ClickEvent>,
}

enum Msg {
    Click,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let request = Request::get("https://api.github.com/repos/betta-cyber/notebook/contents")
            .body(Nothing)
            .expect("error");
        let mut a = FetchService::default();
        a.fetch(request,
            link.callback(|response: Response<Result<String, failure::Error>>| {
                if response.status().is_success() {
                    Msg::Click
                } else {
                    Msg::Click
                }
            })
        );

        App {
            clicked: false,
            onclick: link.callback(|_| Msg::Click),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Click => {
                self.clicked = true;
                true // Indicate that the Component should re-render
            }
        }
    }

    fn view(&self) -> Html {
        let button_text = if self.clicked { "Clicked!" } else { "Click me!" };

        html! {
            <button onclick=&self.onclick>{ button_text }</button>
        }
    }
}

fn main() {
    yew::start_app::<App>();
}
