use leptos::prelude::{ClassAttribute, Get, IntoAny, StyleAttribute, Suspense};
use leptos::prelude::{ElementChild, LocalResource};
use leptos::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum NodeType {
    Root,
    Branch,
    ImageLeaf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    pub hash: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<u64>,
    pub src: Option<String>,
    pub storage_path: Option<String>,
    pub thumbnail_url: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchData {
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodeData {
    Branch(BranchData),
    Image(ImageData),
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub node_type: NodeType,
    pub name: Option<String>,
    pub data: serde_json::Value,
    pub path: String,
    pub updated_at: String,
    pub depth: i32,
    pub own: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tree {
    pub id: Uuid,
    pub node_type: NodeType,
    pub name: Option<String>,
    pub data: NodeData,
    pub raw_data: serde_json::Value,
    pub path: String,
    pub updated_at: String,
    pub depth: i32,
    pub own: bool,
    pub children: Vec<Tree>,
}

impl Tree {
    pub fn node_type_str(&self) -> &str {
        match self.node_type {
            NodeType::Root => "Root",
            NodeType::Branch => "Branch",
            NodeType::ImageLeaf => "ImageLeaf",
        }
    }
}

/// Parse raw JSON data into typed NodeData based on node type
fn parse_node_data(node_type: NodeType, raw_data: &serde_json::Value) -> NodeData {
    match node_type {
        NodeType::ImageLeaf => match serde_json::from_value::<ImageData>(raw_data.clone()) {
            Ok(image_data) => NodeData::Image(image_data),
            Err(_) => NodeData::Empty,
        },
        NodeType::Branch | NodeType::Root => {
            match serde_json::from_value::<BranchData>(raw_data.clone()) {
                Ok(branch_data) => NodeData::Branch(branch_data),
                Err(_) => NodeData::Empty,
            }
        }
    }
}

/// Converts a flat list of TreeNodes into a hierarchical tree structure
/// Root nodes are discarded, and their children become top-level nodes
pub fn build_tree(nodes: Vec<TreeNode>) -> Vec<Tree> {
    // Create a HashMap for quick lookup by id
    let mut node_map: HashMap<Uuid, TreeNode> =
        nodes.into_iter().map(|node| (node.id, node)).collect();

    // Find and remove the root node(s)
    let root_ids: Vec<Uuid> = node_map
        .values()
        .filter(|node| node.node_type == NodeType::Root)
        .map(|node| node.id)
        .collect();

    // Remove root nodes from the map
    for root_id in &root_ids {
        node_map.remove(root_id);
    }

    // Build parent-to-children mapping
    let mut children_map: HashMap<Option<Uuid>, Vec<Uuid>> = HashMap::new();

    for node in node_map.values() {
        children_map
            .entry(node.parent_id)
            .or_default()
            .push(node.id);
    }

    // Helper function to recursively build tree
    fn build_subtree(
        node_id: Uuid,
        node_map: &HashMap<Uuid, TreeNode>,
        children_map: &HashMap<Option<Uuid>, Vec<Uuid>>,
    ) -> Tree {
        let node = node_map.get(&node_id).unwrap();

        let children = children_map
            .get(&Some(node_id))
            .map(|child_ids| {
                child_ids
                    .iter()
                    .map(|&child_id| build_subtree(child_id, node_map, children_map))
                    .collect()
            })
            .unwrap_or_default();

        let parsed_data = parse_node_data(node.node_type, &node.data);

        Tree {
            id: node.id,
            node_type: node.node_type,
            name: node.name.clone(),
            data: parsed_data,
            raw_data: node.data.clone(),
            path: node.path.clone(),
            updated_at: node.updated_at.clone(),
            depth: node.depth,
            own: node.own,
            children,
        }
    }

    // Find all nodes that were children of root nodes
    let top_level_ids: Vec<Uuid> = root_ids
        .iter()
        .filter_map(|root_id| children_map.get(&Some(*root_id)))
        .flatten()
        .copied()
        .collect();

    // Build trees for each top-level node
    top_level_ids
        .into_iter()
        .map(|node_id| build_subtree(node_id, &node_map, &children_map))
        .collect()
}

/// Fetch tree data from API and convert to hierarchical structure
pub async fn fetch_tree_data(user_id: &str, with_leafs: bool) -> Result<Vec<Tree>, String> {
    // Get window object
    let window = web_sys::window().ok_or_else(|| "No window available".to_string())?;

    // Build URL
    let url = format!("/api/proxy/tree/{}", user_id);
    let url = if with_leafs {
        format!("{}?with_leafs=true", url)
    } else {
        url
    };

    logging::log!("Fetching from URL: {}", url);

    // Create request options
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    // Create request
    let request = Request::new_with_str_and_init(&url, &opts)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;

    // Set headers
    request
        .headers()
        .set("Accept", "application/json")
        .map_err(|e| format!("Failed to set header: {:?}", e))?;

    // Fetch the request
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;

    // Convert to Response
    let resp: Response = resp_value
        .dyn_into()
        .map_err(|_| "Failed to convert to Response".to_string())?;

    // Check status
    if !resp.ok() {
        return Err(format!("Request failed with status: {}", resp.status()));
    }

    // Get response body as JSON
    let json = JsFuture::from(
        resp.json()
            .map_err(|e| format!("Failed to get JSON: {:?}", e))?,
    )
    .await
    .map_err(|e| format!("Failed to parse JSON: {:?}", e))?;

    // Deserialize to Vec<TreeNode>
    let nodes: Vec<TreeNode> = serde_wasm_bindgen::from_value(json)
        .map_err(|e| {
            leptos::logging::error!("Deserialize error: {:?}", e);
            format!("Failed to deserialize: {:?}", e)
        })?;

    logging::log!("✓ Received {} nodes", nodes.len());

    // Convert flat nodes to hierarchical tree
    let tree = build_tree(nodes);

    logging::log!("✓ Built tree with {} root nodes", tree.len());

    Ok(tree)
}

/// TreeViewer component with customizable renderer
#[component]
pub fn TreeViewerResource<F, IV>(
    user_id: String,
    with_leafs: bool,
    /// Custom renderer component that receives Vec<Tree>
    renderer: F,
) -> impl IntoView
where
    F: Fn(Vec<Tree>) -> IV + 'static + Send,
    IV: IntoView,
{
    tracing::info!("TreeViewerResource created for user: {}", user_id);
    // Use LocalResource instead of Resource for non-Send futures
    let tree_resource = LocalResource::new(move || {
        tracing::info!("LocalResource fetcher called");
        let user_id = user_id.clone();
        async move {
            tracing::info!("Starting fetch_tree_data for: {}", user_id);
            fetch_tree_data(&user_id, with_leafs).await
        }
    });
    view! {
        <div class="tree-viewer-resource">
            <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                {move || {
                    tree_resource.get().map(|result| {
                        match result {
                            Ok(tree) => renderer(tree).into_any(),
                            Err(_e) => view! {
                                <div class="error">
                                    <p>"✗ Error loading tree!"</p>
                                </div>
                            }.into_any(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

// ============================================================================
// Example renderer components
// ============================================================================

/// Simple list renderer
#[component]
pub fn SimpleTreeRenderer(tree: Vec<Tree>) -> impl IntoView {
    view! {
        <div class="simple-tree">
            <h3>"Simple Tree View"</h3>
            <p>"Total root nodes: " {tree.len()}</p>
            <ul>
                {tree.into_iter().map(|node| {
                    view! {
                        <li>
                            {node.name.unwrap_or_else(|| "(unnamed)".to_string())}
                            " (" {node.children.len()} " children)"
                        </li>
                    }
                }).collect::<Vec<_>>()}
            </ul>
        </div>
    }
}

/// Hierarchical tree renderer
#[component]
pub fn HierarchicalTreeRenderer(tree: Vec<Tree>) -> impl IntoView {
    view! {
        <div class="hierarchical-tree">
            <h3>"Hierarchical Tree View"</h3>
            {tree.into_iter().map(|node| {
                view! { <TreeNodeView node=node depth=0 /> }
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
fn TreeNodeView(node: Tree, depth: usize) -> impl IntoView {
    let indent_style = format!("margin-left: {}px;", depth * 20);
    let icon = match node.node_type {
        NodeType::Root => "🌳",
        NodeType::Branch => "📁",
        NodeType::ImageLeaf => "🖼️",
    };

    let has_children = !node.children.is_empty();
    let children = node.children.clone();

    view! {
        <div class="tree-node" style={indent_style}>
            <div class="node-header">
                <span class="icon">{icon}</span>
                <span class="name">{node.name.clone().unwrap_or_else(|| "(unnamed)".to_string())}</span>
                <span class="meta">" [" {node.node_type_str()} ", depth: " {node.depth} "]"</span>
            </div>
            {has_children.then(|| {
                view! {
                    <div class="children">
                        {children.into_iter().map(|child| {
                            view! { <TreeNodeView node=child depth={depth + 1} /> }
                        }).collect::<Vec<_>>()}
                    </div>
                }
            })}
        </div>
    }
}

/// Card-based tree renderer
#[component]
pub fn CardTreeRenderer(tree: Vec<Tree>) -> impl IntoView {
    view! {
        <div class="card-tree">
            <h3>"Card View"</h3>
            <div class="card-grid">
                {tree.into_iter().map(|node| {
                    view! {
                        <div class="card">
                            <h4>{node.name.clone().unwrap_or_else(|| "(unnamed)".to_string())}</h4>
                            <p>"Type: " {node.node_type_str()}</p>
                            <p>"Children: " {node.children.len()}</p>
                            <p>"Updated: " {node.updated_at}</p>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

// ============================================================================
// Usage examples
// ============================================================================

/// Example 1: Using simple renderer
#[component]
pub fn ExampleSimple() -> impl IntoView {
    view! {
        <TreeViewerResource
            user_id="shpirkov@gmail.com".to_string()
            with_leafs=false
            renderer=|tree| view! { <SimpleTreeRenderer tree=tree /> }
        />
    }
}

/// Example 2: Using hierarchical renderer
#[component]
pub fn ExampleHierarchical() -> impl IntoView {
    view! {
        <TreeViewerResource
            user_id="shpirkov@gmail.com".to_string()
            with_leafs=true
            renderer=|tree| view! { <HierarchicalTreeRenderer tree=tree /> }
        />
    }
}

/// Example 3: Using card renderer
#[component]
pub fn ExampleCards() -> impl IntoView {
    view! {
        <TreeViewerResource
            user_id="shpirkov@gmail.com".to_string()
            with_leafs=false
            renderer=|tree| view! { <CardTreeRenderer tree=tree /> }
        />
    }
}

/// Example 4: Using inline custom renderer
#[component]
pub fn ExampleCustom() -> impl IntoView {
    view! {
        <TreeViewerResource
            user_id="shpirkov@gmail.com".to_string()
            with_leafs=false
            renderer=|tree| view! {
                <div class="custom-view">
                    <h3>"My Custom View"</h3>
                    <p>"Found " {tree.len()} " root nodes"</p>
                    <pre>{format!("{:#?}", tree)}</pre>
                </div>
            }
        />
    }
}
