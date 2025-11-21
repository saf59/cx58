use crate::auth::Auth;
use crate::components::home_page::HomePage;
use crate::components::side_body::SideBody;
use crate::components::side_top::SideTop;
use crate::components::sidebar::SideBar;
use crate::components::user_info::UserRolesDisplay;
use crate::server_fn::*;
use leptos::IntoView;
use leptos::prelude::*;
use leptos_meta::{Link, MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::*;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    provide_meta_context();
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <Title text="CX58 AI agent" />
                <AutoReload options=options.clone() />
                <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico" />
                <Stylesheet id="leptos" href="/pkg/cx58-client.css" />
                <Stylesheet href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css" />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}
/// Auth init is root for all role base auth
/// rest is only with use_context::<RwSignal<Auth>>() like in RootPage
#[component]
pub fn App() -> impl IntoView {
    let initial_auth_resource = Resource::new(|| (), |_| async { get_auth().await });
    let auth_signal = RwSignal::new(Auth::Unauthenticated);
    provide_context(auth_signal);
    Effect::new(move |_| {
        if let Some(Ok(auth)) = initial_auth_resource.get() {
            auth_signal.set(auth);
        }
    });

    view! {
        
        <Suspense fallback=move || {
            view! { <p>"Loading authentication status..."</p> }
        }>
            <Router>
                <main>
                    <Routes fallback=|| view! { <NotFoundPage /> }>
                        <Route path=path!("/") view=move || { RootPage } />
                        <Route path=path!("/profile") view=ProfilePage />
                    </Routes>
                </main>
            </Router>
        </Suspense>
    }
}
#[component]
fn RootPage() -> impl IntoView {
    // 1. Retrieve the Auth state from context.
    // This assumes the context provides an RwSignal<Auth>.
    // Using expect() is common here, but you should ensure the context is set up.
    let auth_signal = use_context::<RwSignal<Auth>>()
        .expect("Auth context not found. Did you set up the provider?");

    view! {
        // You don't typically need Suspense/ErrorBoundary here anymore
        // if the initial Auth state is provided synchronously via context.
        // The context should handle initial loading/fetching before rendering this page.
        // If the context itself is handling async loading, the logic below still works.

        {move || {
            let auth = auth_signal.get();
            if !auth.is_authenticated() {
                // User is not authenticated
                view! { <LoginPage /> }
                    .into_any()
            } else if auth.is_authenticated_guest() {
                // User is authenticated but is a GUEST
                view! { <PublicLandingPage /> }
                    .into_any()
            } else {
                // User is authenticated and is NOT a guest (e.g., a user or admin)
                view! { <SideBar top=SideTop() side_body=SideBody() content=HomePage() /> }
                    .into_any()
            }
        }}
    }
}

// Dummy components
#[component]
fn PublicLandingPage() -> impl IntoView {
    view! {
        <div class="centered  bg_oidc">
            <h1>"Welcome! You are authenticated!"</h1>
            <p>
                <span>"This is the public"</span>
                <span class="cx58">"Construct-X/5.8"</span>
                <span>"home page."</span>
            </p>
            <p>"Unfortunately, you do not have access to any objects."</p>
            <p>"Please contact the administrator to gain access!"</p>
            <LogoutButton />
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
