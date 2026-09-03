// NOTE: DEBUG THIS ONE 喔
// NOTE: DEBUG THIS ONE 㒼

use std::collections::{HashMap, HashSet};
use leptos::either::either;
use leptos::logging::log;
use leptos::{prelude::*, task::spawn_local};
use leptos_meta::*;
use crate::model::hanzi_strokes::HanziStrokes;
use crate::{components::nav::Nav, model::{error::ArcZhongCharError, hanzi::Hanzi}};

#[derive(Debug, Clone, PartialEq)]
struct DecompNode {
    id: Vec<u32>, // The unique structural ID for rendering
    path: Vec<u32>, // The component path for matching strokes
    char: char,
    children: Vec<DecompNode>,
}

const LEGEND_COLORS: &[&str] = &[
    "#e6194B", "#3cb44b", "#ffe119", "#4363d8", "#f58231", 
    "#911eb4", "#46f0f0", "#f032e6", "#bcf60c", "#fabebe", 
    "#008080", "#e6beff", "#9A6324", "#fffac8", "#800000",
];
const DEFAULT_COLOR: &str = "#a9a9a9";


#[component]
fn NodeView(
    node: DecompNode,
    selected_node_id: Signal<Option<Vec<u32>>>,
    on_select: WriteSignal<Option<DecompNode>>,
) -> impl IntoView {
    let is_positional = "⿰⿱⿲⿳⿴⿵⿶⿷⿸⿹⿺⿻".contains(node.char);
    let is_unknown = "？".contains(node.char);
    let node_id = node.id.clone();
    let is_selected = move || selected_node_id.get() == Some(node_id.clone());
    let node_children_clone = node.children.clone();
    view! {
        <li>
            <span
                class:text-accent=is_positional
                class:font-mono=true
                class:text-lg=true
                class:bg-primary=is_selected.clone()
                class:text-primary-content=is_selected
                class="cursor-pointer hover:bg-base-300 rounded-md px-1"
                on:click=move |_| {
                    if !is_positional && !is_unknown {
                        on_select.set(Some(node.clone()));
                    }
                }
            >
                {node.char.to_string()}
            </span>

            {either!(!node_children_clone.is_empty(),
                true => view! {
                    <ul class="pl-4">
                        {node_children_clone.into_iter()
                            .map(|child| view! { <NodeView node=child selected_node_id=selected_node_id on_select=on_select /> })
                            .collect::<Vec<_>>()}
                    </ul>
                },
                false => view! {}
            )}
        </li>
    }
}

/// Helper function to get the number of children for a positional character.
fn get_arity(c: char) -> usize {
    match c {
        '⿰' | '⿱' | '⿴' | '⿵' | '⿶' | '⿷' | '⿸' | '⿹' | '⿺' | '⿻' => 2,
        '⿲' | '⿳' => 3,
        _ => 0, // Not a positional character.
    }
}

#[component]
fn StrokeOrderView(
    character: char,
    strokes: Vec<String>,
    matches: Vec<Option<Vec<u32>>>,
    sub_components: Vec<SubComponent>, // The new prop
) -> impl IntoView {
    // Create a mapping from a component's path to its assigned color.
    let path_to_color: HashMap<Vec<u32>, &str> = sub_components
        .iter()
        .enumerate()
        .map(|(i, sc)| {
            // Assign a color from the predefined list, cycling if necessary.
            let color = LEGEND_COLORS[i % LEGEND_COLORS.len()];
            (sc.path.clone(), color)
        })
        .collect();

    view! {
        <div class="p-4 border bg-base-100 rounded-box">
            <h3 class="text-xl font-bold mb-2">"Strokes for " <span class="font-mono">{character}</span></h3>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 1024 1024"
                class="w-64 h-64 bg-base-300 rounded"
            >
                <g transform="scale(1, -1) translate(0, -900)">
                    {strokes.into_iter()
                        .enumerate()
                        .map(|(i, d)| {
                            // Default to a muted color for strokes not part of any known component.
                            let mut color = DEFAULT_COLOR;
                            if let Some(Some(match_path)) = matches.get(i) {
                                // If the stroke's path is in our map, use the assigned color.
                                if let Some(c) = path_to_color.get(match_path) {
                                    color = *c;
                                }
                            }
                            view! { <path fill=color d=d></path> }
                        })
                        .collect::<Vec<_>>()}
                </g>
            </svg>

            // --- Render the Legend ---
            <div class="mt-4">
                <h4 class="text-lg font-bold mb-2">"Components"</h4>
                <div class="space-y-2">
                    <For
                        each=move || sub_components.clone()
                        key=|sc| sc.path.clone()
                        children=move |sc| {
                            let color = path_to_color.get(&sc.path).cloned().unwrap_or(DEFAULT_COLOR);
                            view! {
                                <div class="flex items-center gap-2">
                                    <div class="w-4 h-4 rounded" style:background-color=color></div>
                                    <span class="font-mono text-lg">{sc.char}</span>
                                </div>
                            }
                        }
                    />
                    // Add a legend item for the default/unassigned color
                    <div class="flex items-center gap-2">
                        <div class="w-4 h-4 rounded" style:background-color=DEFAULT_COLOR></div>
                        <span class="font-mono text-lg">"Other"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[derive(Debug, Clone, PartialEq)]
struct SubComponent {
    char: char,
    path: Vec<u32>,
}

fn get_immediate_sub_components(node: &DecompNode) -> Vec<SubComponent> {
    fn collect_recursive(node: &DecompNode, collection: &mut Vec<SubComponent>) {
        let is_positional = "⿰⿱⿲⿳⿴⿵⿶⿷⿸⿹⿺⿻".contains(node.char);
        
        if is_positional {
            for child in &node.children {
                collect_recursive(child, collection);
            }
        } else {
            // It's a component character (e.g., '廿', '丨', '？'). Add it and stop traversing.
            collection.push(SubComponent {
                char: node.char,
                path: node.path.clone(),
            });
        }
    }

    let mut sub_components = Vec::new();
    for child in &node.children {
        collect_recursive(child, &mut sub_components);
    }
    sub_components
}



#[component]
pub fn Decomposition() -> impl IntoView {
    let position_characters = "⿰⿱⿲⿳⿴⿵⿶⿷⿸⿹⿺⿻";
    let unknown_character = "？";

    let (input_char, set_input_char) = signal(String::new());
    let (decomposition_tree, set_decomposition_tree) = signal(Option::<DecompNode>::None);
    let (is_searching, set_is_searching) = signal(false);
    let (error_message, set_error_message) = signal(Option::<String>::None);
    let (selected_node, set_selected_node) = signal(Option::<DecompNode>::None);
    let sub_components = Signal::derive(move || {
        selected_node.get().map_or(vec![], |node| get_immediate_sub_components(&node))
    });
    let selected_structural_id = move || selected_node.get().map(|n| n.id);
    let selected_component_path = move || selected_node.get().map(|n| n.path);

    let char_details_resource = LocalResource::new(move || async move {
        // This resource now triggers when the root character of the tree changes.
        if let Some(node) = selected_node.get() {
            let hanzi_data = Hanzi::get_one_from_db(node.char).await.ok().flatten();
            let stroke_data = HanziStrokes::get_one_from_db(node.char).await.ok().flatten();
            (hanzi_data, stroke_data)
        } else {
            (None, None)
        }
    });
    
    let on_submit = move |_| {
        let char_to_search = input_char.get().chars().next();
        if let Some(start_char) = char_to_search {
            spawn_local(async move {
                set_is_searching(true);
                set_decomposition_tree(None);
                set_selected_node(None);
                set_error_message(None);

                match build_tree(start_char).await {
                    Ok(tree) => {
                        set_selected_node(Some(tree.clone()));
                        set_decomposition_tree(Some(tree))
                    },
                    Err(e) => set_error_message(Some(e.to_string())),
                }

                set_is_searching(false);
            });
        }
    };

    let stroke_view = move || {
        Suspend::new(async move {
            // Await the new resource that provides a tuple.
            let details = char_details_resource.await;
            
            // Use either! to handle the combined data.
            either!(details.0.is_some() && details.1.is_some(),
                true => {
                    let hanzi_data = details.0.unwrap();
                    let stroke_data = details.1.unwrap();
                    view! {
                        <StrokeOrderView
                            character=hanzi_data.character
                            strokes=stroke_data.strokes
                            matches=hanzi_data.matches
                            sub_components=sub_components.get()
                        />
                    }
                },
                false => {
                    either!(decomposition_tree.get().is_some(),
                        true => {
                            view! { <div class="p-4 border bg-base-100 rounded-box"><p>"No stroke or decomposition data found."</p></div> }
                        },
                        false => {
                            view! { <div class="p-4 border bg-base-100 rounded-box"><p>"Select a component to see its strokes."</p></div> }
                        }
                    )
                }
            )
        })
    };

    Effect::new(move |_| {
        spawn_local(async {
            log!("Fetching all Hanzi to find unique etymology types...");
            match Hanzi::get_all_from_db().await {
                Ok(hanzi_list) => {
                    // Use a HashSet to automatically store only unique values.
                    let mut unique_types = HashSet::new();
                    let mut multiple_pinyin = Vec::new();
                    for hanzi in hanzi_list {
                        if let Some(etymology) = hanzi.etymology {
                            // Add the type to the set. Duplicates are ignored.
                            unique_types.insert(etymology.r#type);
                        }
                        if hanzi.pinyin.len() > 1 {
                            // 2. Add the character and its pinyin Vec to the formatted string
                            multiple_pinyin.push(format!(
                                "char: {}, pinyins: {:?}", 
                                hanzi.character, 
                                hanzi.pinyin
                            ));
                        }

                    }
                    // Log the final set of unique types to the browser console.
                    log!("Unique Etymology Types: {:?}", unique_types);

                    if multiple_pinyin.is_empty() {
                        log!("No characters found with multiple pinyin readings.");
                    } else {
                        // 3. Join the vector elements with a newline character for logging
                        let log_output = multiple_pinyin.join("\n");
                        log!("Characters with multiple pinyin readings:\n{}", log_output);
                    }
                }
                Err(e) => {
                    log!("[ERROR] Failed to get all Hanzi for debug: {:?}", e);
                }
            }
        });
    });


    view! {
        <main>
            <Title text="中 Char"/>
            <Nav/>
            <div class="p-6">
                <h1 class="text-2xl font-bold mb-4">"Recursive Decomposition"</h1>
                <p class="mb-4 text-base-content/80">"Enter a single Chinese character to see all of its constituent components, down to the radicals."</p>

                <div class="flex items-center gap-2 mb-6">
                    <input
                        type="text"
                        placeholder="Enter character..."
                        class="input input-bordered w-full max-w-xs"
                        maxlength="1"
                        on:input=move |ev| set_input_char(event_target_value(&ev))
                        prop:value=input_char
                        on:keydown=move |ev| { if ev.key() == "Enter" { on_submit(()) } }
                    />
                    <button class="btn btn-primary" on:click=move |_| on_submit(()) disabled=is_searching>
                        {move || either!(is_searching.get(),
                            true =>  {view! { <span class="loading loading-spinner"></span> "Searching..." }},
                            false => {"Decompose"},
                        )}
                    </button>
                </div>

                <div class="flex flex-col md:flex-row gap-6">
                    <div class="flex-1">
                        {move || decomposition_tree.get().map(|tree| {
                            let root_char = tree.char;
                            view! {
                                <h2 class="text-xl font-bold mb-2">"Decomposition of " <span class="font-mono">{root_char}</span></h2>
                                <ul class="menu bg-base-100 rounded-box p-4">
                                    <NodeView
                                        node=tree
                                        selected_node_id=Signal::derive(selected_structural_id)
                                        on_select=set_selected_node
                                    />
                                </ul>
                            }
                        })}
                    </div>

                    <div class="flex-1">
                         <Suspense fallback=move || view!{<div class="p-4 skeleton w-64 h-72"></div>}>
                            <ErrorBoundary fallback=|_| view!{<p>"Error loading strokes."</p>}>
                                {stroke_view}
                            </ErrorBoundary>
                        </Suspense>
                    </div>
                </div>



                // Display any errors that occur
                {move || error_message.get().map(|err| view! {
                    <div class="p-4 mt-4 text-error-content bg-error/20 border border-error rounded-lg">
                        {err}
                    </div>
                })}
            </div>
        </main>
    }
    
}


/// Top-level entry point. Builds the node for the input character.
async fn build_tree(char: char) -> Result<DecompNode, Box<dyn std::error::Error>> {
    // The root node is a component, and we assign it a base path of [0].
    let mut node = DecompNode { id: vec![0], path: vec![0], char, children: vec![] };

    if let Ok(Some(hanzi)) = Hanzi::get_one_from_db(char).await {
        if !hanzi.decomposition.is_empty() {
            // For the character's main decomposition, the components will have paths
            // relative to an empty root, creating paths like [0], [1], etc.
            node.children = parse_decomposition(&hanzi.decomposition, &node.id, &vec![]).await?;
        }
    }
    //log!("Completed tree: {:?}", node);
    Ok(node)
}

/// Parses an entire decomposition string into a vector of its top-level nodes.
async fn parse_decomposition(
    decomposition: &str,
    parent_id: &[u32],
    // The component path of the character being decomposed.
    container_component_path: &[u32],
) -> Result<Vec<DecompNode>, Box<dyn std::error::Error>> {
    let mut children = Vec::new();
    let mut iter = decomposition.chars();
    let mut structural_child_index = 0;

    // This loop handles sequences of components. A positional operator will consume
    // its required children from the iterator within its own recursive call.
    while iter.clone().next().is_some() {
        if let Some(node) = Box::pin(parse_node_from_iter(
            &mut iter,
            parent_id.to_vec(),
            structural_child_index,
            container_component_path.to_vec(),
            false,
        ))
        .await?
        {
            children.push(node);
            structural_child_index += 1;
        } else {
            break; 
        }
    }
    Ok(children)
}


async fn parse_node_from_iter(
    iter: &mut impl Iterator<Item = char>,
    mut parent_id: Vec<u32>,
    structural_index: u32, // This node's index relative to its siblings in the decomposition string.
    parent_component_path: Vec<u32>, // The path of the component we are currently inside.
    parent_is_positional: bool,
) -> Result<Option<DecompNode>, Box<dyn std::error::Error>> {
    if let Some(char) = iter.next() {
        parent_id.push(structural_index);
        let current_id = parent_id;

        let arity = get_arity(char);

        // Determine the final path for THIS specific node.
        let node_path = if arity > 0 {
            // It's a positional node.It only gets a path if it's not the first in a sequence.
            if !parent_is_positional {
                vec![]
            } else {
                let mut path = parent_component_path.clone();
                path.push(structural_index);
                path
            }
        } else {
            // It's a component node. Its path is its parent's context plus its own index.
            let mut path = parent_component_path.clone();
            path.push(structural_index);
            path
        };

        let mut node = DecompNode {
            id: current_id.clone(),
            path: node_path.clone(), // Use the calculated path
            char,
            children: vec![],
        };

        // Now, recursively parse children if any exist.
        if arity > 0 {
            // This is a positional node. It consumes children from the current decomposition string.
            let mut children = Vec::new();
            for i in 0..arity {
                // *** THE FIX: A positional operator passes ITS OWN PATH as the context for its children. ***
                if let Some(child) =
                    Box::pin(parse_node_from_iter(iter, node.id.clone(), i as u32, node_path.clone(), true)).await?
                {
                    children.push(child);
                }
            }
            node.children = children;
        } else { 
            // This is a component node. Check if it has its own decomposition from the database.
            if let Ok(Some(hanzi)) = Hanzi::get_one_from_db(char).await {
                if !hanzi.decomposition.is_empty() {
                    // A sub-decomposition always starts with a fresh, empty path context.
                    node.children =
                        parse_decomposition(&hanzi.decomposition, &node.id, &vec![]).await?;
                }
            }
        }
        
        return Ok(Some(node));
    }
    Ok(None)
}
