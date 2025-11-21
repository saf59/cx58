use crate::components::user_info::UserRolesDisplay;
use crate::server_fn::*;
use leptos::prelude::*;
use leptos::IntoView;
use leptos_meta::{provide_meta_context, Link, MetaTags, Stylesheet, Title};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::*;
use leptos_router::hooks::use_location;
use crate::auth::Auth;
use crate::components::home_page::HomePage;
use crate::components::side_body::SideBody;
use crate::components::side_top::SideTop;
use crate::components::sidebar::SideBar;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    provide_meta_context();
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <Title text="CX58 AI agent" />
                // <AutoReload options=options.clone() />
                <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico" />
                <Stylesheet id="leptos" href="/pkg/cx58-client.css" />
                <Stylesheet href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css" />
                // TODO - HydrationScripts add ruins /login /logout routes
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
    let is_authenticated = Resource::new(|| (), |_| get_is_authenticated());
    //let location = use_location();
    //let auth = leptos::context::use_context::<Auth>().expect("to have found the Auth provided");
    view! {
        <Router>
            <main>
                // <div>{move || use_location().pathname.get()}</div>
                <Routes fallback=|| view! { <NotFoundPage /> }>
                    <Route path=path!("/") view=move || view! { <RootPage is_authenticated /> } />
                    <Route path=path!("/profile") view=ProfilePage />
                </Routes>
            </main>
        </Router>
    }
}

// *** New RootPage Component ***
#[component]
fn RootPage(is_authenticated: Resource<Result<bool, ServerFnError>>) -> impl IntoView {
    view! {
        // 3. Use <Suspense> for initial loading and <Transition> for smooth transitions
        <Suspense fallback=move || view! { <h1>"Loading..."</h1> }>
            // 4. Use <ErrorBoundary> to handle server function errors
            <ErrorBoundary fallback=|errors| {
                view! {
                    <h1>"Error loading auth status."</h1>
                    <p>{format!("{:?}", errors.get())}</p>
                }
            }>
                {move || match is_authenticated.get() {
                    None => {
                        // Resource is still loading or hasn't started
                        view! { <h1>"Checking Auth..."</h1> }
                            .into_any()
                    }
                    Some(Err(_)) => {
                        // Server Function returned an error
                        view! { <LoginPage /> }
                            .into_any()
                    }
                    Some(Ok(true)) => {
                        // Server Function succeeded
                        // User is authenticated, redirect
                        // Use the built-in Leptos <Redirect/> or <Navigate/>
                        view! { <SideBar top=SideTop() side_body=SideBody() content=HomePage() /> }
                            .into_any()
                    }
                    Some(Ok(false)) => {
                        // User is NOT authenticated, show the public landing page or login.
                        view! { <PublicLandingPage /> }
                            .into_any()
                    }
                }}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
fn Chat() -> impl IntoView {
    let auth = use_context::<Auth>();
    view! {
        <div class="centered bg_oidc">
            <h3>"Welcome  to CX58!"</h3>
            <p>"This is the public chat page."</p>
            <LogoutButton />
        </div>
        <div class="centered bg_oidc">
            {move || match auth.clone() {
                Some(Auth::Authenticated(user)) => view! { <p>{user.to_string()}</p> }.into_any(),
                Some(Auth::Unauthenticated) | None => view! { <p>Unauthenticated</p> }.into_any(),
            }}
        </div>
    }
}

// Dummy components
#[component]
fn PublicLandingPage() -> impl IntoView {
    view! {
        <div class="centered  bg_oidc">
            <h1>"Welcome! Please Log In."</h1>
            <p>
                <span>"This is the public"</span>
                <span class="cx58">"Construct-X/5.8"</span>
                <span>"home page."</span>
            </p>
            <LoginButton />
        </div>
    }
}
#[component]
fn LoginPage() -> impl IntoView {
    view! {
        <div class="centered bg_oidc">
            <h3>"Welcome  to CX58!"</h3>
            <h3>You are unauthenticated!</h3>
            <LoginButton />
        </div>
    }
}
#[component]
fn ProfilePage() -> impl IntoView {
    view! {
        <div class="centered  bg_oidc">
            <UserRolesDisplay />
            <LogoutButton />
        </div>
    }
}
#[component]
pub fn LoginButton() -> impl IntoView {
    view! {
        // it is axum route - not leptos
        <a href="/login" class="sign" rel="external">
            <i class="fa fa-sign-in"></i>
            <span>Log In</span>
        </a>
    }
}
#[component]
pub fn LogoutButton() -> impl IntoView {
    view! {
        // it is axum route - not leptos
        <a href="/logout" class="sign sign-out" rel="external">
            <i class="fa fa-sign-out"></i>
            <span>Log Out</span>
        </a>
    }
}

#[component]
fn NotFoundPage() -> impl IntoView {
    // set an HTTP status code 404
    // this is feature gated because it can only be done during
    // initial server-side rendering
    // if you navigate to the 404 page subsequently, the status
    // code will not be set because there is not a new HTTP request
    // to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_axum::ResponseOptions>();
        resp.set_status(reqwest::StatusCode::NOT_FOUND);
    }
    view! {
        <h1>"404 - Page Not Found"</h1>
        <p>"The requested page was not found."</p>
    }
}
