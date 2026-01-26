#![allow(unused_variables)]
#![allow(dead_code)]
use leptos::prelude::{ClassAttribute, Get, IntoAny, Suspense};
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageData {
    pub hash: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<u64>,
    pub src: Option<String>,
    pub storage_path: Option<String>,
    pub thumbnail_url: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchData {
    pub title: Option<String>,
}

// #[serde(tag = "type", rename_all = "snake_case")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NodeData {
    Image(ImageData),
    Branch(BranchData),
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tree {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeWithLeaf {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub node_type: NodeType,
    pub name: Option<String>,
    pub data: NodeData,
    pub path: String,
    pub updated_at: String,
    pub full_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: Option<String>,
    pub node_type: NodeType,
}

impl Tree {
    pub fn node_type_str(&self) -> &str {
        match self.node_type {
            NodeType::Root => "Root",
            NodeType::Branch => "Branch",
            NodeType::ImageLeaf => "ImageLeaf",
        }
    }
    pub fn node_info(&self) -> NodeInfo {
        NodeInfo {
            id: self.id,
            parent_id: self.parent_id,
            name: self.name.clone(),
            node_type: self.node_type,
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
            parent_id: node.parent_id,
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
            logging::error!("Deserialize error: {:?}", e);
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
            <Suspense fallback=move || {
                view! { <i class="tree-loader"></i> }
            }>
                {move || {
                    tree_resource
                        .get()
                        .map(|result| {
                            match result {
                                Ok(tree) => renderer(tree).into_any(),
                                Err(_e) => {
                                    view! {
                                        <div class="error">
                                            <p>"✗ Error loading tree!"</p>
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


