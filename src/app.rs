use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
pub use crate::components::home_page::HomePage;
use crate::components::side_body::SideBody;
use crate::components::side_top::SideTop;
use crate::components::sidebar::SideBar;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/cx58.css" />
        <Stylesheet href="https://use.fontawesome.com/releases/v5.6.1/css/all.css" />

        // sets the document title
        <Title text="CX58 AI agent" />

        // content for this welcome page
        <Router>
            <main style="display: flex;height: 100%;width: 100%;">
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route
                        path=StaticSegment("")
                        view=|| {
                            view! {
                                <SideBar top=SideTop() side_body=SideBody() content=HomePage() />
                            }
                        }
                    />
                </Routes>
            </main>
        </Router>
    }
}
