use crate::auth::AuthenticatedUser;
use leptos::IntoView;
use leptos::prelude::*;

#[component]
pub fn UserRolesDisplay(user: Option<AuthenticatedUser>) -> impl IntoView {
    if let Some(info) = user {
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
                                let role_class = format!("role-badge role-{}", role.as_str());
                                view! { <span class=role_class>{role.as_str()}</span> }
                            })
                            .collect::<Vec<_>>()}
                    </div>
                </div>

            </div>
        }
        .into_any()
    } else {
        ().into_any()
    }
}
