# Model Settings Sidebar Plan

## Goal

Add model settings controls to the `gmr` sidebar using the `cx58-agent`
model-settings API.

## Agent Startup Assumption

For local verification, `cx58-agent` is started separately:

1. Start SSH with all required port forwards.
2. Run `D:\projects\rust\cx\cx58-agent\run_it_gate.cmd`.

`local_run_it.cmd` is for starting `gmr`, not the agent.

## Implementation Steps

1. Add shared Rust structs for the model settings API response/request.
2. Add server-side `gmr` proxy routes:
   - `GET /api/models/{user_id}`
   - `PUT /api/models/{user_id}`
3. Keep HMAC signing on the server side only; the browser calls only local
   `/api/models/...` routes.
4. Add a sidebar component that loads current settings and available models.
5. Render selectors for:
   - `vision_model`
   - `text_model`
   - `chat_model`
6. Add a `same` checkbox beside `vision_model`.
7. When `same` is checked, send `vision_model` and `same=true`; the agent decides
   which roles can actually use that model.
8. When `same` is unchecked, send all explicit roles and `same=false`.
9. After save, update UI from the agent response `current` settings and show
   `changes` including rejected role updates.
10. Add compact sidebar styling and localization keys.
11. Verify with `cargo fmt`, SSR check, and a local run against the separately
   started agent.

## Boundary

The UI does not enforce model compatibility. It only filters/display candidates
and sends the user's intent. The agent remains the source of truth for whether a
model can be applied to a role.
