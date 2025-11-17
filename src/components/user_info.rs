use leptos::prelude::*;
use leptos::IntoView;
use crate::server_fn::get_auth;

#[component]
pub fn UserRolesDisplay() -> impl IntoView {
    let auth = Resource::new(|| (), |_| async { get_auth().await });

    view! {
        <div class="user-roles-container">
            <Suspense fallback=move || {
                view! { <div class="loading">"Loading user info..."</div> }
            }>
                {move || {
                    auth
                        .get()
                        .map(|result| {
                            match result {
                                Ok(info) => {
                                    view! {
                                        <div class="user-info-card">
                                            <h2>"User Profile"</h2>
                                            <div class="user-subject">
                                                <strong>"User ID: "</strong>
                                                <span>{info.name}</span>
                                            </div>

                                            <div class="roles-section">
                                                <h3>"Roles"</h3>
                                                <div class="roles-list">
                                                    {info
                                                        .roles
                                                        .iter()
                                                        .map(|role| {
                                                            let role_class = format!(
                                                                "role-badge role-{}",
                                                                role.as_str(),
                                                            );
                                                            view! { <span class=role_class>{role.as_str()}</span> }
                                                        })
                                                        .collect::<Vec<_>>()}
                                                </div>
                                            </div>

                                        </div>
                                    }
                                        .into_any()
                                }
                                Err(e) => {

                                    view! {
                                        <div class="error">
                                            <p>"Error loading user info: " {e.to_string()}</p>
                                        </div>
                                    }
                                        .into_any()
                                }
                            }
                        })
                }}
            </Suspense>

        </div>
    }
}
